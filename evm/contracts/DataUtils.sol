//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.4;

// By putting these helper functions in a separate DataUtils contract,
// the DataAggregator contract can call these functions in try/catch blocks.
contract DataUtils {
    function decodeBytesAsString (bytes memory data) external pure returns (string memory) {
        return abi.decode(data, (string));
    }

    function decodeBytesAsBytes32 (bytes memory data) external pure returns (bytes32) {
        return abi.decode(data, (bytes32));
    }

    function bytes32ToString (bytes32 data) external pure returns (string memory) {
        uint8 i = 0;
        while (i < 32 && data[i] != 0) {
            i++;
        }

        bytes memory bytesArray = new bytes(i);
        for (i = 0; i < 32 && data[i] != 0; i++) {
            bytesArray[i] = data[i];
        }

        return string(bytesArray);
    }
}