SELECT SUM(Transactions.amount - (
    SELECT COALESCE(SUM(amount), 0)
    FROM Splits
    WHERE Splits.transactionId = Transactions.id
)) + (
           SELECT COALESCE(SUM(amount), 0)
           FROM Splits
           WHERE Splits.categoryId = 2
       )
FROM Transactions
WHERE Transactions.categoryId = 2;
