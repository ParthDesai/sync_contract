// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/// @notice ERC20 credit token used by SyncContract. Only `mintAuthority` can mint and can
/// transfer mint authority once (or multiple times) by calling `transferMintAuthority`.
contract StrovaCreditToken is ERC20 {
    error NotMintAuthority();
    error ZeroAddress();

    event MintAuthorityTransferred(address indexed oldAuthority, address indexed newAuthority);

    address public mintAuthority;
    uint8 private immutable _decimals;

    constructor(
        string memory name_,
        string memory symbol_,
        uint8 decimals_,
        address initialMintAuthority
    ) ERC20(name_, symbol_) {
        if (initialMintAuthority == address(0)) revert ZeroAddress();
        mintAuthority = initialMintAuthority;
        _decimals = decimals_;
    }

    function decimals() public view override returns (uint8) {
        return _decimals;
    }

    function mint(address to, uint256 amount) external {
        if (msg.sender != mintAuthority) revert NotMintAuthority();
        _mint(to, amount);
    }

    function transferMintAuthority(address newAuthority) external {
        if (msg.sender != mintAuthority) revert NotMintAuthority();
        if (newAuthority == address(0)) revert ZeroAddress();
        address old = mintAuthority;
        mintAuthority = newAuthority;
        emit MintAuthorityTransferred(old, newAuthority);
    }
}


