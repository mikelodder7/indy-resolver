use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde_json;
use serde_json::Value;

use api::ledger::{CustomFree, CustomTransactionParser};
use domain::ledger::request::Request;
use errors::prelude::*;
use services::crypto::CryptoService;
use services::ledger::LedgerService;
use services::pool::{
    PoolService,
    parse_response_metadata
};
use services::wallet::{RecordOptions, WalletService};
use utils::crypto::base58;
use utils::crypto::signature_serializer::serialize_signature;
use api::WalletHandle;
use commands::{Command, CommandExecutor};

pub enum LedgerCommand {
    SignAndSubmitRequest(
        i32, // pool handle
        WalletHandle,
        String, // submitter did
        String, // request json
        Box<Fn(IndyResult<String>) + Send>),
    SubmitRequest(
        i32, // pool handle
        String, // request json
        Box<Fn(IndyResult<String>) + Send>),
    SubmitAck(
        i32, // cmd_id
        IndyResult<String>, // result json or error
    ),
    SubmitAction(
        i32, // pool handle
        String, // request json
        Option<String>, // nodes
        Option<i32>, // timeout
        Box<Fn(IndyResult<String>) + Send>),
    SignRequest(
        WalletHandle,
        String, // submitter did
        String, // request json
        Box<Fn(IndyResult<String>) + Send>),
    MultiSignRequest(
        WalletHandle,
        String, // submitter did
        String, // request json
        Box<Fn(IndyResult<String>) + Send>),
    BuildNodeRequest(
        String, // submitter did
        String, // target_did
        NodeOperationData, // data
        Box<Fn(IndyResult<String>) + Send>),
    BuildGetValidatorInfoRequest(
        String, // submitter did
        Box<Fn(IndyResult<String>) + Send>),
    BuildGetTxnRequest(
        Option<String>, // submitter did
        Option<String>, // ledger type
        i32, // data
        Box<Fn(IndyResult<String>) + Send>),
    BuildPoolConfigRequest(
        String, // submitter did
        bool, // writes
        bool, // force
        Box<Fn(IndyResult<String>) + Send>),
    BuildPoolRestartRequest(
        String, //submitter did
        String, //action
        Option<String>, //datetime
        Box<Fn(IndyResult<String>) + Send>),
    BuildPoolUpgradeRequest(
        String, // submitter did
        String, // name
        String, // version
        String, // action
        String, // sha256
        Option<u32>, // timeout
        Option<String>, // schedule
        Option<String>, // justification
        bool, // reinstall
        bool, // force
        Option<String>, // package
        Box<Fn(IndyResult<String>) + Send>),
    BuildAuthRuleRequest(
        String, // submitter did
        String, // auth type
        String, // auth action
        String, // field
        Option<String>, // old value
        Option<String>, // new value
        String, // constraint
        Box<Fn(IndyResult<String>) + Send>),
    BuildAuthRulesRequest(
        String, // submitter did
        AuthRules, // auth rules
        Box<Fn(IndyResult<String>) + Send>),
    BuildGetAuthRuleRequest(
        Option<String>, // submitter did
        Option<String>, // auth type
        Option<String>, // auth action
        Option<String>, // field
        Option<String>, // old value
        Option<String>, // new value
        Box<Fn(IndyResult<String>) + Send>),
    RegisterSPParser(
        String, // txn type
        CustomTransactionParser,
        CustomFree,
        Box<Fn(IndyResult<()>) + Send>),
    GetResponseMetadata(
        String, // response
        Box<Fn(IndyResult<String>) + Send>),
    BuildTxnAuthorAgreementRequest(
        String, // submitter did
        String, // text
        String, // version
        Box<Fn(IndyResult<String>) + Send>),
    BuildGetTxnAuthorAgreementRequest(
        Option<String>, // submitter did
        Option<GetTxnAuthorAgreementData>, // data
        Box<Fn(IndyResult<String>) + Send>),
    BuildAcceptanceMechanismRequests(
        String, // submitter did
        AcceptanceMechanisms, // aml
        String, // version
        Option<String>, // aml context
        Box<Fn(IndyResult<String>) + Send>),
    BuildGetAcceptanceMechanismsRequest(
        Option<String>, // submitter did
        Option<u64>, // timestamp
        Option<String>, // version
        Box<Fn(IndyResult<String>) + Send>),
    AppendTxnAuthorAgreementAcceptanceToRequest(
        String, // request json
        Option<String>, // text
        Option<String>, // version
        Option<String>, // hash
        String, // acceptance mechanism type
        u64, // time of acceptance
        Box<Fn(IndyResult<String>) + Send>),
}

pub struct LedgerCommandExecutor {
    pool_service: Rc<PoolService>,
    crypto_service: Rc<CryptoService>,
    wallet_service: Rc<WalletService>,
    ledger_service: Rc<LedgerService>,

    send_callbacks: RefCell<HashMap<i32, Box<Fn(IndyResult<String>)>>>,
    pending_callbacks: RefCell<HashMap<i32, Box<Fn(IndyResult<(String, String)>)>>>,
}

impl LedgerCommandExecutor {
    pub fn new(pool_service: Rc<PoolService>,
               crypto_service: Rc<CryptoService>,
               wallet_service: Rc<WalletService>,
               ledger_service: Rc<LedgerService>) -> LedgerCommandExecutor {
        LedgerCommandExecutor {
            pool_service,
            crypto_service,
            wallet_service,
            ledger_service,
            send_callbacks: RefCell::new(HashMap::new()),
            pending_callbacks: RefCell::new(HashMap::new()),
        }
    }

    pub fn execute(&self, command: LedgerCommand) {
        match command {
            LedgerCommand::SignAndSubmitRequest(pool_handle, wallet_handle, submitter_did, request_json, cb) => {
                info!(target: "ledger_command_executor", "SignAndSubmitRequest command received");
                self.sign_and_submit_request(pool_handle, wallet_handle, &submitter_did, &request_json, cb);
            }
            LedgerCommand::SubmitRequest(handle, request_json, cb) => {
                info!(target: "ledger_command_executor", "SubmitRequest command received");
                self.submit_request(handle, &request_json, cb);
            }
            LedgerCommand::SubmitAck(handle, result) => {
                info!(target: "ledger_command_executor", "SubmitAck command received");
                match self.send_callbacks.borrow_mut().remove(&handle) {
                    Some(cb) => cb(result.map_err(IndyError::from)),
                    None => {
                        error!("Can't process LedgerCommand::SubmitAck for handle {} with result {:?} - appropriate callback not found!",
                               handle, result);
                    }
                }
            }
            LedgerCommand::SubmitAction(handle, request_json, nodes, timeout, cb) => {
                info!(target: "ledger_command_executor", "SubmitRequest command received");
                self.submit_action(handle, &request_json, nodes.as_ref().map(String::as_str), timeout, cb);
            }
            LedgerCommand::RegisterSPParser(txn_type, parser, free, cb) => {
                info!(target: "ledger_command_executor", "RegisterSPParser command received");
                cb(self.register_sp_parser(&txn_type, parser, free));
            }
            LedgerCommand::SignRequest(wallet_handle, submitter_did, request_json, cb) => {
                info!(target: "ledger_command_executor", "SignRequest command received");
                cb(self.sign_request(wallet_handle, &submitter_did, &request_json));
            }
            LedgerCommand::MultiSignRequest(wallet_handle, submitter_did, request_json, cb) => {
                info!(target: "ledger_command_executor", "MultiSignRequest command received");
                cb(self.multi_sign_request(wallet_handle, &submitter_did, &request_json));
            }
            LedgerCommand::BuildGetTxnRequest(submitter_did, ledger_type, seq_no, cb) => {
                info!(target: "ledger_command_executor", "BuildGetTxnRequest command received");
                cb(self.build_get_txn_request(submitter_did.as_ref().map(String::as_str), ledger_type.as_ref().map(String::as_str), seq_no));
            }
            LedgerCommand::BuildPoolConfigRequest(submitter_did, writes, force, cb) => {
                info!(target: "ledger_command_executor", "BuildPoolConfigRequest command received");
                cb(self.build_pool_config_request(&submitter_did, writes, force));
            }
            LedgerCommand::BuildPoolRestartRequest(submitter_did, action, datetime, cb) => {
                info!(target: "ledger_command_executor", "BuildPoolRestartRequest command received");
                cb(self.build_pool_restart_request(&submitter_did, &action, datetime.as_ref().map(String::as_str)));
            }
            LedgerCommand::BuildPoolUpgradeRequest(submitter_did, name, version, action, sha256, timeout, schedule, justification, reinstall, force, package, cb) => {
                info!(target: "ledger_command_executor", "BuildPoolUpgradeRequest command received");
                cb(self.build_pool_upgrade_request(&submitter_did, &name, &version, &action, &sha256, timeout,
                                                   schedule.as_ref().map(String::as_str),
                                                   justification.as_ref().map(String::as_str),
                                                   reinstall, force, package.as_ref().map(String::as_str)));
            }
            LedgerCommand::GetResponseMetadata(response, cb) => {
                info!(target: "ledger_command_executor", "GetResponseMetadata command received");
                cb(self.get_response_metadata(&response));
            }
            LedgerCommand::BuildAuthRuleRequest(submitter_did, txn_type, action, field, old_value, new_value, constraint, cb) => {
                info!(target: "ledger_command_executor", "BuildAuthRuleRequest command received");
                cb(self.build_auth_rule_request(&submitter_did, &txn_type, &action, &field, old_value.as_ref().map(String::as_str), new_value.as_ref().map(String::as_str), &constraint));
            }
            LedgerCommand::BuildAuthRulesRequest(submitter_did, rules, cb) => {
                info!(target: "ledger_command_executor", "BuildAuthRulesRequest command received");
                cb(self.build_auth_rules_request(&submitter_did, rules));
            }
            LedgerCommand::BuildGetAuthRuleRequest(submitter_did, txn_type, action, field, old_value, new_value, cb) => {
                info!(target: "ledger_command_executor", "BuildGetAuthRuleRequest command received");
                cb(self.build_get_auth_rule_request(submitter_did.as_ref().map(String::as_str),
                                                    txn_type.as_ref().map(String::as_str),
                                                    action.as_ref().map(String::as_str),
                                                    field.as_ref().map(String::as_str),
                                                    old_value.as_ref().map(String::as_str),
                                                    new_value.as_ref().map(String::as_str)));
            }
            LedgerCommand::BuildTxnAuthorAgreementRequest(submitter_did, text, version, cb) => {
                info!(target: "ledger_command_executor", "BuildTxnAuthorAgreementRequest command received");
                cb(self.build_txn_author_agreement_request(&submitter_did, &text, &version));
            }
            LedgerCommand::BuildGetTxnAuthorAgreementRequest(submitter_did, data, cb) => {
                info!(target: "ledger_command_executor", "BuildGetTxnAuthorAgreementRequest command received");
                cb(self.build_get_txn_author_agreement_request(submitter_did.as_ref().map(String::as_str), data.as_ref()));
            }
            LedgerCommand::BuildAcceptanceMechanismRequests(submitter_did, aml, version, aml_context, cb) => {
                info!(target: "ledger_command_executor", "BuildAcceptanceMechanismRequests command received");
                cb(self.build_acceptance_mechanisms_request(&submitter_did, aml, &version, aml_context.as_ref().map(String::as_str)));
            }
            LedgerCommand::BuildGetAcceptanceMechanismsRequest(submitter_did, timestamp, version, cb) => {
                info!(target: "ledger_command_executor", "BuildGetAcceptanceMechanismsRequest command received");
                cb(self.build_get_acceptance_mechanisms_request(submitter_did.as_ref().map(String::as_str),
                                                                timestamp,
                                                                version.as_ref().map(String::as_str)));
            }
            LedgerCommand::AppendTxnAuthorAgreementAcceptanceToRequest(request_json, text, version, hash, acc_mech_type, time_of_acceptance, cb) => {
                info!(target: "ledger_command_executor", "AppendTxnAuthorAgreementAcceptanceToRequest command received");
                cb(self.append_txn_author_agreement_acceptance_to_request(&request_json,
                                                                          text.as_ref().map(String::as_str),
                                                                          version.as_ref().map(String::as_str),
                                                                          hash.as_ref().map(String::as_str),
                                                                          &acc_mech_type,
                                                                          time_of_acceptance));
            }
        };
    }

    fn register_sp_parser(&self, txn_type: &str,
                          parser: CustomTransactionParser, free: CustomFree) -> IndyResult<()> {
        debug!("register_sp_parser >>> txn_type: {:?}, parser: {:?}, free: {:?}",
               txn_type, parser, free);

        PoolService::register_sp_parser(txn_type, parser, free)
            .map_err(IndyError::from)
    }

    fn sign_and_submit_request(&self,
                               pool_handle: i32,
                               wallet_handle: WalletHandle,
                               submitter_did: &str,
                               request_json: &str,
                               cb: Box<Fn(IndyResult<String>) + Send>) {
        debug!("sign_and_submit_request >>> pool_handle: {:?}, wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}",
               pool_handle, wallet_handle, submitter_did, request_json);

        match self._sign_request(wallet_handle, submitter_did, request_json, SignatureType::Single) {
            Ok(signed_request) => self.submit_request(pool_handle, signed_request.as_str(), cb),
            Err(err) => cb(Err(err))
        }
    }

    fn _sign_request(&self,
                     wallet_handle: WalletHandle,
                     submitter_did: &str,
                     request_json: &str,
                     signature_type: SignatureType) -> IndyResult<String> {
        debug!("_sign_request >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

        let my_did: Did = self.wallet_service.get_indy_object(wallet_handle, &submitter_did, &RecordOptions::id_value())?;

        let my_key: Key = self.wallet_service.get_indy_object(wallet_handle, &my_did.verkey, &RecordOptions::id_value())?;

        let mut request: Value = serde_json::from_str(request_json)
            .to_indy(IndyErrorKind::InvalidStructure, "Message is invalid json")?;

        if !request.is_object() {
            return Err(err_msg(IndyErrorKind::InvalidStructure, "Message isn't json object"));
        }

        let serialized_request = serialize_signature(request.clone())?;
        let signature = self.crypto_service.sign(&my_key, &serialized_request.as_bytes().to_vec())?;

        match signature_type {
            SignatureType::Single => { request["signature"] = Value::String(base58::encode(&signature)); }
            SignatureType::Multi => {
                request.as_object_mut()
                    .map(|request| {
                        if !request.contains_key("signatures") {
                            request.insert("signatures".to_string(), Value::Object(serde_json::Map::new()));
                        }
                        request["signatures"].as_object_mut().unwrap().insert(submitter_did.to_string(), Value::String(base58::encode(&signature)));

                        if let (Some(identifier), Some(signature)) = (request.get("identifier").and_then(Value::as_str).map(str::to_owned),
                                                                      request.remove("signature")) {
                            request["signatures"].as_object_mut().unwrap().insert(identifier, signature);
                        }
                    });
            }
        }

        let res: String = serde_json::to_string(&request)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize message after signing")?;

        debug!("_sign_request <<< res: {:?}", res);

        Ok(res)
    }

    fn submit_request(&self,
                      handle: i32,
                      request_json: &str,
                      cb: Box<Fn(IndyResult<String>) + Send>) {
        debug!("submit_request >>> handle: {:?}, request_json: {:?}", handle, request_json);

        let x: IndyResult<i32> = self.pool_service.send_tx(handle, request_json);
        match x {
            Ok(cmd_id) => { self.send_callbacks.borrow_mut().insert(cmd_id, cb); }
            Err(err) => { cb(Err(err)); }
        };
    }

    fn submit_action(&self,
                     handle: i32,
                     request_json: &str,
                     nodes: Option<&str>,
                     timeout: Option<i32>,
                     cb: Box<Fn(IndyResult<String>) + Send>) {
        debug!("submit_action >>> handle: {:?}, request_json: {:?}, nodes: {:?}, timeout: {:?}", handle, request_json, nodes, timeout);

        if let Err(err) = self.ledger_service.validate_action(request_json) {
            return cb(Err(err));
        }

        let x: IndyResult<i32> = self.pool_service.send_action(handle, request_json, nodes, timeout);
        match x {
            Ok(cmd_id) => { self.send_callbacks.borrow_mut().insert(cmd_id, cb); }
            Err(err) => { cb(Err(err)); }
        };
    }

    fn sign_request(&self,
                    wallet_handle: WalletHandle,
                    submitter_did: &str,
                    request_json: &str) -> IndyResult<String> {
        debug!("sign_request >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

        let res = self._sign_request(wallet_handle, submitter_did, request_json, SignatureType::Single)?;

        debug!("sign_request <<< res: {:?}", res);

        Ok(res)
    }

    fn multi_sign_request(&self,
                          wallet_handle: WalletHandle,
                          submitter_did: &str,
                          request_json: &str) -> IndyResult<String> {
        debug!("multi_sign_request >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

        let res = self._sign_request(wallet_handle, submitter_did, request_json, SignatureType::Multi)?;

        debug!("multi_sign_request <<< res: {:?}", res);

        Ok(res)
    }
    

    fn build_node_request(&self,
                          submitter_did: &str,
                          target_did: &str,
                          data: NodeOperationData) -> IndyResult<String> {
        debug!("build_node_request >>> submitter_did: {:?}, target_did: {:?}, data: {:?}",
               submitter_did, target_did, data);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_node_request(submitter_did, target_did, data)?;

        debug!("build_node_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_validator_info_request(&self,
                                        submitter_did: &str) -> IndyResult<String> {
        info!("build_get_validator_info_request >>> submitter_did: {:?}", submitter_did);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_get_validator_info_request(submitter_did)?;

        info!("build_get_validator_info_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_txn_request(&self,
                             submitter_did: Option<&str>,
                             ledger_type: Option<&str>,
                             seq_no: i32) -> IndyResult<String> {
        debug!("build_get_txn_request >>> submitter_did: {:?}, ledger_type: {:?}, seq_no: {:?}",
               submitter_did, ledger_type, seq_no);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_txn_request(submitter_did, ledger_type, seq_no)?;

        debug!("build_get_txn_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_pool_config_request(&self,
                                 submitter_did: &str,
                                 writes: bool,
                                 force: bool) -> IndyResult<String> {
        debug!("build_pool_config_request >>> submitter_did: {:?}, writes: {:?}, force: {:?}",
               submitter_did, writes, force);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_pool_config(submitter_did, writes, force)?;

        debug!("build_pool_config_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn build_pool_restart_request(&self, submitter_did: &str, action: &str,
                                  datetime: Option<&str>) -> IndyResult<String> {
        debug!("build_pool_restart_request >>> submitter_did: {:?}, action: {:?}, datetime: {:?}", submitter_did, action, datetime);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_pool_restart(submitter_did, action, datetime)?;

        debug!("build_pool_config_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn build_pool_upgrade_request(&self,
                                  submitter_did: &str,
                                  name: &str,
                                  version: &str,
                                  action: &str,
                                  sha256: &str,
                                  timeout: Option<u32>,
                                  schedule: Option<&str>,
                                  justification: Option<&str>,
                                  reinstall: bool,
                                  force: bool,
                                  package: Option<&str>) -> IndyResult<String> {
        debug!("build_pool_upgrade_request >>> submitter_did: {:?}, name: {:?}, version: {:?}, action: {:?}, sha256: {:?},\
         timeout: {:?}, schedule: {:?}, justification: {:?}, reinstall: {:?}, force: {:?}, package: {:?}",
               submitter_did, name, version, action, sha256, timeout, schedule, justification, reinstall, force, package);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_pool_upgrade(submitter_did, name, version, action, sha256,
                                                         timeout, schedule, justification, reinstall, force, package)?;

        debug!("build_pool_upgrade_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn get_response_metadata(&self,
                             response: &str) -> IndyResult<String> {
        debug!("get_response_metadata >>> response: {:?}", response);

        let metadata = parse_response_metadata(response)?;

        let res = serde_json::to_string(&metadata)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize ResponseMetadata")?;

        debug!("get_response_metadata <<< res: {:?}", res);

        Ok(res)
    }

    fn build_auth_rule_request(&self,
                               submitter_did: &str,
                               txn_type: &str,
                               action: &str,
                               field: &str,
                               old_value: Option<&str>,
                               new_value: Option<&str>,
                               constraint: &str) -> IndyResult<String> {
        debug!("build_auth_rule_request >>> submitter_did: {:?}, txn_type: {:?}, action: {:?}, field: {:?}, \
            old_value: {:?}, new_value: {:?}, constraint: {:?}", submitter_did, txn_type, action, field, old_value, new_value, constraint);

        self.validate_opt_did(Some(submitter_did))?;

        let res = self.ledger_service.build_auth_rule_request(submitter_did, txn_type, action, field, old_value, new_value, constraint)?;

        debug!("build_auth_rule_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_auth_rules_request(&self,
                               submitter_did: &str,
                               rules: AuthRules) -> IndyResult<String> {
        debug!("build_auth_rules_request >>> submitter_did: {:?}, rules: {:?}", submitter_did, rules);

        self.validate_opt_did(Some(submitter_did))?;

        let res = self.ledger_service.build_auth_rules_request(submitter_did, rules)?;

        debug!("build_auth_rules_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_auth_rule_request(&self,
                                   submitter_did: Option<&str>,
                                   txn_type: Option<&str>,
                                   action: Option<&str>,
                                   field: Option<&str>,
                                   old_value: Option<&str>,
                                   new_value: Option<&str>) -> IndyResult<String> {
        debug!("build_get_auth_rule_request >>> submitter_did: {:?}, auth_type: {:?}, auth_action: {:?}, field: {:?}, \
            old_value: {:?}, new_value: {:?}", submitter_did, txn_type, action, field, old_value, new_value);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_auth_rule_request(submitter_did, txn_type, action, field, old_value, new_value)?;

        debug!("build_get_auth_rule_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_txn_author_agreement_request(&self,
                                          submitter_did: &str,
                                          text: &str,
                                          version: &str) -> IndyResult<String> {
        debug!("build_txn_author_agreement_request >>> submitter_did: {:?}, text: {:?}, version: {:?}", submitter_did, text, version);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_txn_author_agreement_request(submitter_did, text, version)?;

        debug!("build_txn_author_agreement_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_txn_author_agreement_request(&self,
                                              submitter_did: Option<&str>,
                                              data: Option<&GetTxnAuthorAgreementData>) -> IndyResult<String> {
        debug!("build_get_txn_author_agreement_request >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_txn_author_agreement_request(submitter_did, data)?;

        debug!("build_get_txn_author_agreement_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_acceptance_mechanisms_request(&self,
                                           submitter_did: &str,
                                           aml: AcceptanceMechanisms,
                                           version: &str,
                                           aml_context: Option<&str>) -> IndyResult<String> {
        debug!("build_acceptance_mechanisms_request >>> submitter_did: {:?}, aml: {:?}, version: {:?}, aml_context: {:?}", submitter_did, aml, version, aml_context);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_acceptance_mechanisms_request(submitter_did, aml, version, aml_context)?;

        debug!("build_acceptance_mechanisms_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_acceptance_mechanisms_request(&self,
                                               submitter_did: Option<&str>,
                                               timestamp: Option<u64>,
                                               version: Option<&str>) -> IndyResult<String> {
        debug!("build_get_acceptance_mechanisms_request >>> submitter_did: {:?}, timestamp: {:?}, version: {:?}", submitter_did, timestamp, version);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_acceptance_mechanisms_request(submitter_did, timestamp, version)?;

        debug!("build_get_acceptance_mechanisms_request <<< res: {:?}", res);

        Ok(res)
    }

    fn append_txn_author_agreement_acceptance_to_request(&self,
                                                         request_json: &str,
                                                         text: Option<&str>,
                                                         version: Option<&str>,
                                                         taa_digest: Option<&str>,
                                                         acc_mech_type: &str,
                                                         time: u64) -> IndyResult<String> {
        debug!("append_txn_author_agreement_acceptance_to_request >>> request_json: {:?}, text: {:?}, version: {:?}, taa_digest: {:?}, acc_mech_type: {:?}, time: {:?}",
               request_json, text, version, taa_digest, acc_mech_type, time);

        let mut request: Request<serde_json::Value> = serde_json::from_str(request_json)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Cannot deserialize request: {:?}", err)))?;

        request.taa_acceptance = Some(self.ledger_service.prepare_acceptance_data(text, version, taa_digest, acc_mech_type, time)?);

        let res: String = serde_json::to_string(&request)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize request after adding author agreement acceptance data")?;

        debug!("append_txn_author_agreement_acceptance_to_request <<< res: {:?}", res);

        Ok(res)
    }

    fn validate_opt_did(&self, did: Option<&str>) -> IndyResult<()> {
        match did {
            Some(did) => Ok(self.crypto_service.validate_did(did)?),
            None => Ok(())
        }
    }
}

enum SignatureType {
    Single,
    Multi
}
