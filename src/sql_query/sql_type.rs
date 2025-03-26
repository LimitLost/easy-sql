pub enum SqlRangeType {
    ///int4range
    I32,
    ///int8range
    I64,
    ///daterange
    NaiveDate,
    ///tsrange
    NaiveDateTime,
    ///numrange
    BigDecimal,
    ///numrange
    Decimal,
}

pub enum SqlType {
    ///Postgresql: inet
    ///Sqlite: BLOB
    IpAddr,
    ///Postgresql: boolean
    ///Sqlite: BOOLEAN
    Bool,
    ///Postgresql: float4
    ///Sqlite: FLOAT
    F32,
    ///Postgresql: float8
    ///Sqlite: DOUBLE
    F64,
    ///Postgresql: char
    ///Sqlite: INT
    I8,
    ///Postgresql: smallint
    ///Sqlite: INT
    I16,
    ///Postgresql: integer
    ///Sqlite: INT
    I32,
    ///Postgresql: bigint
    ///Sqlite: INT
    I64,
    ///Postgresql: text
    ///Sqlite: TEXT
    String,
    ///Aka Duration or TimeDelta
    ///Postgresql: interval
    ///Sqlite: BLOB
    Interval,
    /// Vec<u8>
    ///Postgresql: bytea
    ///Sqlite: BLOB
    Bytes,
    ///Postgresql: type[]
    ///Sqlite: BLOB
    List(Box<SqlType>),
    ///Postgresql: type[x]
    ///Sqlite: BLOB
    Array {
        data_type: Box<SqlType>,
        size: usize,
    },
    ///Postgresql: date
    ///Sqlite: BLOB
    NaiveDate,
    ///Postgresql: timestamp
    ///Sqlite: BLOB
    NaiveDateTime,
    ///Postgresql: time
    ///Sqlite: BLOB
    NaiveTime,
    ///Postgresql: uuid
    ///Sqlite: BLOB
    Uuid,
    ///Postgresql: NUMERIC
    ///Sqlite: BLOB
    Decimal,
    ///Postgresql: NUMERIC
    ///Sqlite: BLOB
    BigDecimal,
    ///Postgresql: <See TableFieldRangeType>
    ///Sqlite: BLOB
    Range(SqlRangeType),
    //
    //Not Implemented:
    //
    //PgCube
    //IpNetwork
    //Oid
    //PgCiText
    //PgHstore
    //PgInterval
    //PgLQuery
    //PgLTree
    //PgLine
    //PgMoney
    //PgPoint
    //PgRange<Date>
    //PgRange<OffsetDateTime>
    //PgRange<PrimitiveDateTime>
    //PgTimeTz
    //PgTimeTz<NaiveTime, FixedOffset>
    //MacAddress
    //BitVec
    //Date
    //OffsetDateTime
    //PrimitiveDateTime
    //Time
}
