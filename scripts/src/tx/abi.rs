use alloy::sol;

sol! {
#[sol(rpc)]
interface IContentConsumptionContract {
    function initialize(address owner) external;

    function getNonceForPlatform(address user, bytes32 platform_id) external view returns (uint256);

    function pushCcu(address user, bytes32 platform_id, uint256 added_consumption, uint256 deadline, uint8 v, bytes32 r, bytes32 s) external;

    function registerPlatform(string calldata name, address owner, bytes4 content_type, bytes32 origin) external returns (bytes32);

    function domainSeparator() external view returns (bytes32);

    function updatePlatformMetadata(bytes32 platform_id, string calldata name, address owner) external;

    function getPlatformMetadata(bytes32 platform_id) external view returns (string memory, address, bytes4, bytes32);

    function getPlatformName(bytes32 platform_id) external view returns (string memory);

}

}
