use alloy::sol;

sol! {
#[sol(rpc)]
interface IContentConsumptionContract {
    function initialize(address owner) external;

    function getNonceForPlatform(address user, bytes32 platform_id) external view returns (uint256);

    function pushCcu(address user, bytes32 platform_id, uint256 added_consumption, uint256 deadline, uint8 v, bytes32 r, bytes32 s) external;

    function registerPlatform(string calldata name, string calldata origin, address owner, bytes32 content_type, uint256 deadline, uint8 v, bytes32 r, bytes32 s) external returns (bytes32);

    function domainSeparator() external view returns (bytes32);

    function updatePlatformMetadata(bytes32 platform_id, string calldata name, address owner) external;

    function getPlatformMetadata(bytes32 platform_id) external view returns (address, bytes32);

    function getPlatformName(bytes32 platform_id) external view returns (string memory);

    function getPlatforOrigin(bytes32 platform_id) external view returns (string memory);

}

}

// 0x
// 0000000000000000000000000000000000000000000000000000000000000020
// 0000000000000000000000000000000000000000000000000000000000000004
// 7465737400000000000000000000000000000000000000000000000000000000

// 0x -> output test with string
// 0000000000000000000000000000000000000000000000000000000000000020
// 0000000000000000000000007caf754c934710d7c73bc453654552beca38223f -> owner
// 0000000000000000000000000000000000000000000000000000000000000000 -> content type
// 276f72518cc65e824f6bcf3d303e2e3b11bf7193bfc907c7f3b3320ee4b02d39 -> origin
// 0000000000000000000000000000000000000000000000000000000000000080 -> name offset -> wtf??
// 0000000000000000000000000000000000000000000000000000000000000004
// 7465737400000000000000000000000000000000000000000000000000000000
