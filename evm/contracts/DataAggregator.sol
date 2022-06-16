//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.4;

import "./@uniswap/v2-core/contracts/interfaces/IUniswapV2Pair.sol";
import "./@uniswap/v2-core/contracts/interfaces/IUniswapV2ERC20.sol";

// Similar to multicall, this contract aggregates multiple function calls into one.
contract DataAggregator {
    // Given a pair address, return metadata for both tokens in the pair.
    function getTokenMetadata (address pairAddress) external view returns (
        address token0_address, 
        string memory token0_name, 
        string memory token0_symbol, 
        uint8 token0_decimals, 
        address token1_address, 
        string memory token1_name, 
        string memory token1_symbol, 
        uint8 token1_decimals) {
            IUniswapV2Pair pair = IUniswapV2Pair(pairAddress);
            
            token0_address = pair.token0();
            token1_address = pair.token1();

            IUniswapV2ERC20 token0 = IUniswapV2ERC20(token0_address);
            IUniswapV2ERC20 token1 = IUniswapV2ERC20(token1_address);

            token0_name = token0.name();
            token0_symbol = token0.symbol();
            token0_decimals = token0.decimals();

            token1_name = token1.name();
            token1_symbol = token1.symbol();
            token1_decimals = token1.decimals();
        }
}
