use crate::RpcMethod;

pub struct NvimEval;

impl RpcMethod for NvimEval {
	const METHOD: &'static str = "nvim_eval";

	type Params = [String; 1];
	type Response = rmpv::Value;
}
