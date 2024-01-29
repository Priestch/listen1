use serde::Serialize;

pub fn to_query_string<S>(a: &S) -> String
    where S: Serialize
{
    serde_qs::to_string(a).unwrap()
}
