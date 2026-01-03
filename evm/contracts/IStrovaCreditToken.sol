// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IStrovaCreditToken {
    function decimals() external view returns (uint8);

    function mintAuthority() external view returns (address);

    function mint(address to, uint256 amount) external;

    function transferMintAuthority(address newAuthority) external;
}


