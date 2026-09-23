// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IRoyalties} from "./interfaces/Iroyalties.sol";

/**
 * @title Royalties
 * @notice Distributes license payment strictly in USDC (80% creator, 20% treasury)
 * @dev Streaming revenue is handled separately via x402 facilitator
 */
contract Royalties is IRoyalties {
    using SafeERC20 for IERC20;

    uint256 public constant BASIS_POINTS = 10000;
    uint256 public constant TREASURY_FEE_BPS = 2000;  // 20%
    uint256 public constant CREATOR_SHARE_BPS = 8000; // 80%

    address private _admin;
    address private _treasury;
    address private _usdc;

    modifier onlyAdmin() {
        if (msg.sender != _admin) revert NotAdmin(msg.sender);
        _;
    }

    constructor(address treasuryAddress, address adminAddress) {
        if (treasuryAddress == address(0) || adminAddress == address(0)) revert InvalidAddress();
        _treasury = treasuryAddress;
        _admin = adminAddress;
    }

    /// @inheritdoc IRoyalties
    function distributeLicensePayment(
        uint256 songId,
        address payer,
        address creator,
        uint256 licensePrice
    ) external returns (uint256 creatorShare, uint256 treasuryFee) {
        if (creator == address(0) || payer == address(0)) revert InvalidAddress();
        if (licensePrice == 0) {
            emit LicensePaymentDistributed(songId, payer, creator, 0, 0, 0);
            return (0, 0);
        }

        if (_usdc == address(0)) revert UsdcNotSet();

        treasuryFee = (licensePrice * TREASURY_FEE_BPS) / BASIS_POINTS;
        creatorShare = licensePrice - treasuryFee;

        // Distribute strictly from contract balance (funded by payer via caller)
        IERC20(_usdc).safeTransfer(creator, creatorShare);
        IERC20(_usdc).safeTransfer(_treasury, treasuryFee);

        emit LicensePaymentDistributed(songId, payer, creator, licensePrice, creatorShare, treasuryFee);
    }

    /// @inheritdoc IRoyalties
    function calculateSplit(uint256 licensePrice) external pure returns (uint256 creatorShare, uint256 treasuryFee) {
        treasuryFee = (licensePrice * TREASURY_FEE_BPS) / BASIS_POINTS;
        creatorShare = licensePrice - treasuryFee;
    }
}
