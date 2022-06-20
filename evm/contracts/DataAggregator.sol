//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.4;

import "./DataUtils.sol";

// Similar to multicall, this contract aggregates multiple function calls into one.
contract DataAggregator {
    address owner;
    address utils;

    constructor (address _utils) {
        owner = msg.sender;
        utils = _utils;
    }

    // Allow the owner to change the utils contract address.
    function setUtilsAddress (address _utils) external {
        require (msg.sender == owner);
        utils = _utils;
    }

    // Given a pair address, return metadata for both tokens in the pair.
    function getMetadata (address pairAddress) external view returns (
        address factoryAddress,
        address token0Address, 
        string memory token0Name, 
        string memory token0Symbol, 
        uint8 token0Decimals, 
        address token1Address, 
        string memory token1Name, 
        string memory token1Symbol, 
        uint8 token1Decimals) 
    {
        (factoryAddress, token0Address, token1Address) = getPairMetadata(pairAddress);
        (token0Name, token0Symbol, token0Decimals) = getTokenMetadata(token0Address);
        (token1Name, token1Symbol, token1Decimals) = getTokenMetadata(token1Address);
    }

    function getPairMetadata (address pairAddress) private view returns (
        address factoryAddress,
        address token0Address,
        address token1Address)
    {
        (, bytes memory f) = pairAddress.staticcall(abi.encodeWithSignature("factory()"));
        (, bytes memory t0) = pairAddress.staticcall(abi.encodeWithSignature("token0()"));
        (, bytes memory t1) = pairAddress.staticcall(abi.encodeWithSignature("token1()"));

        factoryAddress = abi.decode(f, (address));
        token0Address = abi.decode(t0, (address));
        token1Address = abi.decode(t1, (address));
    }

    function getTokenMetadata (address tokenAddress) private view returns (
        string memory tokenName,
        string memory tokenSymbol,
        uint8 tokenDecimals)
    {
        (, bytes memory n) = tokenAddress.staticcall(abi.encodeWithSignature("name()"));
        (, bytes memory s) = tokenAddress.staticcall(abi.encodeWithSignature("symbol()"));
        (, bytes memory d) = tokenAddress.staticcall(abi.encodeWithSignature("decimals()"));

        DataUtils dataUtils = DataUtils(utils);

        try dataUtils.decodeBytesAsString(n) returns (string memory name) {
            tokenName = name;
        } catch (bytes memory) {
            tokenName = dataUtils.bytes32ToString(dataUtils.decodeBytesAsBytes32(n));
        }

        try dataUtils.decodeBytesAsString(s) returns (string memory symbol) {
            tokenSymbol = symbol;
        } catch (bytes memory) {
            tokenSymbol = dataUtils.bytes32ToString(dataUtils.decodeBytesAsBytes32(s));
        }

        tokenDecimals = abi.decode(d, (uint8));
    }
}