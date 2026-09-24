// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {IRemix} from "./interfaces/Iremix.sol";
import {IFactory} from "./interfaces/Ifactory.sol";
import {IRoyalties} from "./interfaces/Iroyalties.sol";

/**
 * @title Remix
 * @notice Coordinates remix rights licensing and creation
 * @dev Records ongoing rights through Factory and mints licensed variations separately
 */
contract Remix is IRemix, ReentrancyGuard {
    using SafeERC20 for IERC20;

    IFactory private immutable _factory;
    IRoyalties private immutable _royalties;

    address private _admin;
    uint256 private _totalRemixes;
    mapping(uint256 => uint256[]) private _remixes;

    error InvalidAddress();
    error NotAdmin(address caller);

    modifier onlyAdmin() {
        if (msg.sender != _admin) revert NotAdmin(msg.sender);
        _;
    }

    constructor(address factoryAddress, address royaltiesAddress, address adminAddress) {
        if (factoryAddress == address(0) || royaltiesAddress == address(0) || adminAddress == address(0)) {
            revert InvalidAddress();
        }
        _factory = IFactory(factoryAddress);
        _royalties = IRoyalties(royaltiesAddress);
        _admin = adminAddress;
    }

    function originalSong(uint256 originalSongId) private view returns (IFactory.Song memory original) {
        if (!_factory.songExists(originalSongId)) revert SongNotFound(originalSongId);
        original = _factory.getSong(originalSongId);
        if (!original.remixable) revert SongNotRemixable(originalSongId);
        if (original.isRemix) revert RemixCannotBeRemixed(originalSongId);
    }

    /// @inheritdoc IRemix
    function buyLicense(uint256 originalSongId) external nonReentrant {
        IFactory.Song memory original = originalSong(originalSongId);
        if (_factory.hasLicense(originalSongId, msg.sender)) return;
        if (original.licenseAmount == 0) revert NoLicensesAvailable(originalSongId);

        _factory.consumeLicense(originalSongId, msg.sender);

        // Distribute USDC license payment if license price > 0
        if (original.licensePrice > 0) {
            address usdcToken = _royalties.usdc();
            IERC20(usdcToken).safeTransferFrom(msg.sender, address(_royalties), original.licensePrice);
            // forge-lint: disable-next-line(unused-return)
            _royalties.distributeLicensePayment(originalSongId, msg.sender, original.creator, original.licensePrice);
        }

        emit LicensePurchased(originalSongId, msg.sender, original.licensePrice);
    }

    /// @inheritdoc IRemix
    function remixSong(
        uint256 originalSongId,
        RemixParams calldata params,
        string calldata metadataUri
    ) external nonReentrant returns (uint256 remixedSongId) {
        originalSong(originalSongId);
        if (!_factory.hasLicense(originalSongId, msg.sender)) revert LicenseRequired(originalSongId, msg.sender);

        // Mint remix NFT to caller
        remixedSongId = _factory.mintRemix(
            params.licenseAmount,
            params.licensePrice,
            params.streamPrice,
            originalSongId,
            msg.sender,
            metadataUri
        );

        _remixes[originalSongId].push(remixedSongId);
        _totalRemixes++;

        // forge-lint: disable-next-line(reentrancy-events)
        emit SongRemixed(originalSongId, remixedSongId, msg.sender, metadataUri);
    }

    /// @inheritdoc IRemix
    function canRemix(uint256 songId) external view returns (bool) {
        return _factory.canRemix(songId);
    }

    /// @inheritdoc IRemix
    function getRemixesOf(uint256 originalSongId) external view returns (uint256[] memory) {
        return _remixes[originalSongId];
    }

    /// @inheritdoc IRemix
    function getRemixCount(uint256 originalSongId) external view returns (uint256) {
        return _remixes[originalSongId].length;
    }

    /// @inheritdoc IRemix
    function totalRemixes() external view returns (uint256) {
        return _totalRemixes;
    }

    /// @inheritdoc IRemix
    function factory() external view returns (address) {
        return address(_factory);
    }

    /// @inheritdoc IRemix
    function royalties() external view returns (address) {
        return address(_royalties);
    }
}
