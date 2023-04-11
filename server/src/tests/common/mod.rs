use std::str::FromStr;
use std::sync::Arc;

use async_rwlock::RwLock;
use rocket::State;
use rsa::{BigUint, RsaPrivateKey};
use sqlx::{PgPool, Pool, Postgres};
use jumpdrive_auth::services::JwtService;
use crate::models::entities::user::user_role::UserRole;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;

pub struct TestApp {
    pool: Arc<RwLock<Pool<Postgres>>>,
    jwt_service: JwtService,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(RwLock::new(pool)),
            jwt_service: JwtService::new(
                RsaPrivateKey::from_components(
                    BigUint::from_str("74997830905646587139816226014144719862265627823949553374295905850158141318656719276313209175746760261055971134897398913479558563360202476134525738215443985213798786134947536321820103185111448036430087812065337288385932817127530120303914818733328961756008475729319280311987156480371871574865965853381575857139")
                        .unwrap(),
                    BigUint::from(65537_u32),
                    BigUint::from_str("45617567685304330426392636489339624454422611989351069566192497290154477430849944314077138980618476651150864051610769685643768298994884588952051505294448065757179269826950503958317081444597140993894151600136581876170515114272651619643521680297034232277886955997006233794078043072977551674990209356559104937817")
                        .unwrap(),
                    vec![
                        BigUint::from_str("9944441522010244646787246177965507622037745736058678614688713158939513831023761630710456709828736517464244174020145137823187668945784719572371232406003647")
                            .unwrap(),
                        BigUint::from_str("7541683536441165394165027564769197112271246852984650020233604313173600283919646980685011266989909442158283534805756431333923018511568340875819132987922637")
                            .unwrap(),
                    ]
                )
                    .unwrap(),
                300,
                "tester",
                "internal-testing"
            ),
        }
    }

    pub fn pool_state(&self) -> &State<Arc<RwLock<Pool<Postgres>>>> {
        State::from(&self.pool)
    }

    pub fn jwt_service(&self) -> &State<JwtService> {
        State::from(&self.jwt_service)
    }

    pub fn alice(&self) -> JwtUserPayload {
        JwtUserPayload {
            uuid: "abc".to_string(),
            username: "alice".to_string(),
            role: UserRole::User,
        }
    }
}
