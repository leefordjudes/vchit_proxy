## Endpoint: POST http://localhost:5000/query

Request Body:
```json
{
  "query": "select * from apvchit",
  "params":[]
}
```

Response:
```json
{
  "data": [
     {
      "AGE": 31,
      "DOB": "1995-05-15 00:00:00",
      "GENDER": "M",
      "HEIGHT": 175.5,
      "ID": 12.0,
      "NAME": "John Doe"
    }
    //...
  ],
  "status": "success"
}
```

<hr/>

Request Body:
```json
{
  "query": "select * from apvchit where id=:1",
  "params":[31]
}
```

Response:
```json
{
  "data": [
    {
      "AGE": 50,
      "DOB": "1990-01-01 00:00:00",
      "GENDER": "M",
      "HEIGHT": 165.5,
      "ID": 31.0,
      "NAME": "Joe1"
    }
  ],
  "status": "success"
}
```

<hr/>

Request Body:
```json
{
  "query": "select * from apvchit where age >= :1 and gender = :2",
  "params":[35, "M"]
}
```

Response:
```json
{
  "data": [
    {
      "AGE": 35,
      "DOB": "1995-05-15 00:00:00",
      "GENDER": "M",
      "HEIGHT": 175.5,
      "ID": 16.0,
      "NAME": "John Doe"
    },
    {
      "AGE": 37,
      "DOB": "1995-05-15 00:00:00",
      "GENDER": "M",
      "HEIGHT": 175.5,
      "ID": 18.0,
      "NAME": "John Doe"
    }
  ],
  "status": "success"
}
```

<hr/>

Request Body:
```json
{
  "query":"select SEQ_APVCHIT.NEXTVAL from dual",
  "params":[]
}
```

Response:
```json
{
  "data": [
    {
      "NEXTVAL": 33.0
    }
  ],
  "status": "success"
}
```

<hr/>

Request Body:
```json
{
  "query":"select SEQ_APVCHIT.CURRVAL from dual",
  "params":[]
}
```

Response:
```json
{
  "data": [
    {
      "CURRVAL": 30.0
    }
  ],
  "status": "success"
}
```

<hr/>

## Endpoint: POST http://localhost:5000/execute

Request Body:
```json
{
  "query": "insert INTO APVCHIT (ID, NAME, DOB, GENDER, AGE, HEIGHT) VALUES (SEQ_APVCHIT.NEXTVAL, :1, TO_DATE(:2, 'YYYY-MM-DD'), :3, :4, :5)",
  "params":["Joe1", "1990-01-01","M",50,165.50]
}
```
Response:
```json
{
  "data": {
    "rows_affected": 1
  },
  "status": "success"
}
```

<hr/>

## TEST SQL
```sql
DROP TABLE APVCHIT;

CREATE TABLE APVCHIT
(
    ID     NUMBER NOT NULL,
    NAME   VARCHAR2(250),
    DOB    DATE,
    GENDER CHAR(1),
    AGE    NUMBER(3),
    HEIGHT NUMBER(8, 2),
    CONSTRAINT PK_APVCHIT PRIMARY KEY (ID)
);

CREATE SEQUENCE SEQ_APVCHIT START WITH 1 INCREMENT BY 1;

INSERT INTO APVCHIT (ID, NAME, DOB, GENDER, AGE, HEIGHT)
VALUES (SEQ_APVCHIT.NEXTVAL, 'John Doe', TO_DATE('1995-05-15', 'YYYY-MM-DD'), 'M', 31, 175.50);

--OR--
CREATE OR REPLACE TRIGGER TRG_APVCHIT_BI
BEFORE INSERT ON APVCHIT
FOR EACH ROW
BEGIN
    IF :NEW.ID IS NULL THEN
        SELECT SEQ_APVCHIT.NEXTVAL INTO :NEW.ID FROM DUAL;
    END IF;
END;

INSERT INTO APVCHIT (NAME, DOB, GENDER, AGE, HEIGHT)
VALUES ('John Doe', TO_DATE('1995-05-15', 'YYYY-MM-DD'), 'M', 31, 175.50);
```