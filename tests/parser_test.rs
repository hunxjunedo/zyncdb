use zyncdb::{commands::{Del, Expire, Get, Set}, constants::{DEL_FIELD_NAME_ERROR, EXPIRE_ARGS_ERROR, EXPIRE_INVALID_TTL_ERROR, GET_FIELD_NAME_ERROR, PARSER_INVALID_COMMAND, SET_ARGS_ERROR, SET_INVALID_TTL_ERROR}, from_string};

#[test]
fn throws_error_on_empty_message(){
    assert!(from_string("    ".into()).is_err_and(|x|x.to_string()==PARSER_INVALID_COMMAND));
}

#[test]
fn throws_error_on_invalid_command(){
    assert!(from_string("GETe".into()).is_err_and(|x|x.to_string()==PARSER_INVALID_COMMAND));
    assert!(from_string("get".into()).is_err_and(|x|x.to_string()==PARSER_INVALID_COMMAND));
    assert!(from_string("EXPP".into()).is_err_and(|x|x.to_string()==PARSER_INVALID_COMMAND));
}

#[test]
fn parses_get(){
   //command parsed correclt 
   assert!(from_string("GET key".into()).unwrap().downcast_ref::<Get>().expect("expected GET").field == "key");
   //missing args check
   assert!(from_string("GET     ".into()).is_err_and(|e|e.to_string()==GET_FIELD_NAME_ERROR));
}


#[test]
fn parses_set(){
   //command parsed correclt: without ttl
   assert!(from_string("SET key value".into()).unwrap().downcast_ref::<Set>().expect("expected SET") == &Set {field: "key".into(), data: "value".into(), ttl: None});
   //with ttl 
   assert!(from_string("SET key value 30".into()).unwrap().downcast_ref::<Set>().expect("expected SET") == &Set {field: "key".into(), data: "value".into(), ttl: Some(30)});
  //wrong ttl format
  assert!(from_string("SET key value thirty".into()).is_err_and(|e|e.to_string() == SET_INVALID_TTL_ERROR));
  //wrong or missing args
  assert!(from_string("SET key ".into()).is_err_and(|v|v.to_string() == SET_ARGS_ERROR));
  assert!(from_string("SET   ".into()).is_err_and(|v|v.to_string() == SET_ARGS_ERROR));

}

#[test]
fn parses_expire(){
    //correct command
    assert!(from_string("EXP key 30".into()).unwrap().downcast_ref::<Expire>().expect("expected EXP") == &Expire {field: "key".into(), ttl: 30});
    //wrong format of ttl
    assert!(from_string("EXP key thirty".into()).is_err_and(|v|v.to_string() == EXPIRE_INVALID_TTL_ERROR));
    //missing args
    assert!(from_string("EXP key ".into()).is_err_and(|v|v.to_string() == EXPIRE_ARGS_ERROR));
    assert!(from_string("EXP      ".into()).is_err_and(|v|v.to_string() == EXPIRE_ARGS_ERROR));
}

#[test]
fn parses_del(){
//command parsed correclt 
   assert!(from_string("DEL key".into()).unwrap().downcast_ref::<Del>().expect("expected DEL").field == "key");
   //missing args check
   assert!(from_string("DEL     ".into()).is_err_and(|e|e.to_string()==DEL_FIELD_NAME_ERROR));
}

#[test]
fn rejects_bad_ttl(){
    assert!(from_string("EXP key 99999999999999999999".into()).is_err_and(|v|v.to_string() == EXPIRE_INVALID_TTL_ERROR));
    assert!(from_string("SET key value 99999999999999999999".into()).is_err_and(|v|v.to_string() == SET_INVALID_TTL_ERROR));
    assert!(from_string("EXP key -999".into()).is_err_and(|v|v.to_string() == EXPIRE_INVALID_TTL_ERROR));
    assert!(from_string("SET key value -999".into()).is_err_and(|v|v.to_string() == SET_INVALID_TTL_ERROR));
    assert!(from_string("EXP key 0".into()).is_err_and(|v|v.to_string() == EXPIRE_INVALID_TTL_ERROR));
    assert!(from_string("SET key value 0".into()).is_err_and(|v|v.to_string() == SET_INVALID_TTL_ERROR));
}
