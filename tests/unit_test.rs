use zyncdb::{ResponseFromDbOp, commands::Get, db::{self, Database}, db_operations::{DbOperation, GetKey, RemoveKey, SetKey}};

#[test]
fn gets_value() {
    let mut db = Database::new();
    db.data.insert("test_key".to_string(), ("test_value".to_string(), None));
    let response = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == "test_value".to_string()));
}

#[test]
fn get_fails_for_non_existant_key(){
    let mut db = Database::new();
    let response = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::Error(x) if x == "Value does not exist!".to_string()));
}

#[test]
fn set_new_value(){
    let mut db = Database::new();
    let response = SetKey {key: "test_key".to_string(), value: "test_value".to_string(), ttl: None}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == "Field did not exist, but created it".to_string()));
    //now check if it actually set the value
    let response_second = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response_second, ResponseFromDbOp::NormalResponse(x) if x == "test_value".to_string()));

}

#[test]
fn update_old_value(){
    let mut db = Database::new();
    SetKey {key: "test_key".to_string(), value: "test_value".to_string(), ttl: None}.execute(&mut db);
    let response = SetKey {key: "test_key".to_string(), value: "test_value2".to_string(), ttl: None}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == "Value updated".to_string()));
    //now check if it actually updates the value
    let response_second = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response_second, ResponseFromDbOp::NormalResponse(x) if x == "test_value2".to_string()));

}

#[test]
fn remove_non_existant_key(){
    let mut db = Database::new();
    let response = RemoveKey {key: "non_existant_key".to_string()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == "the key did not exist".to_string()));
}

#[test]
fn remove_existing_key(){
    let mut db = Database::new();
    SetKey{key: "test_key".to_string(), value: "test_value".to_string(), ttl: None}.execute(&mut db);
    let response = RemoveKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == "key removed succesfully".to_string()));
    //read again to ensure it does not exist
    let response_second = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response_second, ResponseFromDbOp::Error(x) if x == "Value does not exist!".to_string()));
}