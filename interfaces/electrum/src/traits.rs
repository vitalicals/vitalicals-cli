use anyhow::Result;

pub trait ElectrumApi {
	fn shadowsatsGetFtInfo(id: String) -> Result<()>;
}
