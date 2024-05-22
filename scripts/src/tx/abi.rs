use alloy::sol;

sol! {
    function initialize(address owner) external;
    function registerPlatform(string calldata name, address owner, bytes4 content_type, bytes32 origin) external returns (bytes32);
}
