use alloy::sol;

sol! {
#[sol(rpc)]
interface IContentConsumptionContract {
    function initialize(address owner, uint256 frakContentId, address contentRegistryAddress) external;

    function pushCcu(bytes32 channelId, uint256 addedConsumption, uint256 deadline, uint8 v, bytes32 r, bytes32 s) external;

    function getUserConsumption(address user, bytes32 channelId) external returns (uint256);

    function getTotalConsumption() external returns (uint256);

    function domainSeparator() external view returns (bytes32);
}

}
