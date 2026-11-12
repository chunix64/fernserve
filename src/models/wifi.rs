use serde::Serialize;

#[derive(Serialize)]
pub struct AccessPointInfo {
    pub ssid: heapless::String<32>,
    pub auth_method: AuthenticationMethod,
    pub rssi: i8,
    pub channel: u8,
}

#[derive(Serialize)]
pub enum AuthenticationMethod {
    None,
    Wep,
    Wpa,
    Wpa2Personal,
    WpaWpa2Personal,
    Wpa2Enterprise,
    Wpa3Personal,
    Wpa2Wpa3Personal,
    WapiPersonal,
    Owe,
    Wpa3EntSuiteB192Bit,
    Wpa3ExtPsk,
    Wpa3ExtPskMixed,
    Dpp,
    Wpa3Enterprise,
    Wpa2Wpa3Enterprise,
    WpaEnterprise,
    Unknown,
}

impl From<esp_radio::wifi::AuthenticationMethod> for AuthenticationMethod {
    fn from(value: esp_radio::wifi::AuthenticationMethod) -> Self {
        match value {
            esp_radio::wifi::AuthenticationMethod::None => Self::None,
            esp_radio::wifi::AuthenticationMethod::Wep => Self::Wep,
            esp_radio::wifi::AuthenticationMethod::Wpa => Self::Wpa,
            esp_radio::wifi::AuthenticationMethod::Wpa2Personal => Self::Wpa2Personal,
            esp_radio::wifi::AuthenticationMethod::WpaWpa2Personal => Self::WpaWpa2Personal,
            esp_radio::wifi::AuthenticationMethod::Wpa2Enterprise => Self::Wpa2Enterprise,
            esp_radio::wifi::AuthenticationMethod::Wpa3Personal => Self::Wpa3Personal,
            esp_radio::wifi::AuthenticationMethod::Wpa2Wpa3Personal => Self::Wpa2Wpa3Personal,
            esp_radio::wifi::AuthenticationMethod::WapiPersonal => Self::WapiPersonal,
            esp_radio::wifi::AuthenticationMethod::Owe => Self::Owe,
            esp_radio::wifi::AuthenticationMethod::Wpa3EntSuiteB192Bit => Self::Wpa3EntSuiteB192Bit,
            esp_radio::wifi::AuthenticationMethod::Wpa3ExtPsk => Self::Wpa3ExtPsk,
            esp_radio::wifi::AuthenticationMethod::Wpa3ExtPskMixed => Self::Wpa3ExtPskMixed,
            esp_radio::wifi::AuthenticationMethod::Dpp => Self::Dpp,
            esp_radio::wifi::AuthenticationMethod::Wpa3Enterprise => Self::Wpa3Enterprise,
            esp_radio::wifi::AuthenticationMethod::Wpa2Wpa3Enterprise => Self::Wpa2Wpa3Enterprise,
            esp_radio::wifi::AuthenticationMethod::WpaEnterprise => Self::WpaEnterprise,
            _ => Self::Unknown,
        }
    }
}

impl TryFrom<esp_radio::wifi::ap::AccessPointInfo> for AccessPointInfo {
    type Error = heapless::CapacityError;

    fn try_from(value: esp_radio::wifi::ap::AccessPointInfo) -> Result<Self, Self::Error> {
        let ssid: heapless::String<32> = heapless::String::try_from(value.ssid.as_str())?;
        let auth_method = match value.auth_method {
            Some(auth_method) => auth_method.into(),
            None => AuthenticationMethod::Unknown,
        };

        Ok(Self {
            ssid,
            auth_method,
            rssi: value.signal_strength,
            channel: value.channel,
        })
    }
}
