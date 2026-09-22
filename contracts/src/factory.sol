// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";
import {IFactory} from "./interfaces/Ifactory.sol";

/**
 * @title Factory
 * @notice Rawchunks Song Factory and ERC721 registry
 * @dev Tracks song licenses, pricing in USDC, and IPFS metadata URI
 */
contract Factory is ERC721, IFactory {
    uint256 private _totalSongs;
    uint256 private _nextSongId;
    address private _admin;

    mapping(uint256 => Song) private _songs;
    mapping(address => bool) private _authorizedConsumers;
    mapping(uint256 => mapping(address => bool)) private _hasLicense;

    modifier onlyAdmin() {
        if (msg.sender != _admin) revert NotAuthorized(msg.sender);
        _;
    }

    modifier onlyExistingSong(uint256 songId) {
        if (_ownerOf(songId) == address(0)) revert SongNotFound(songId);
        _;
    }

    modifier onlySongCreator(uint256 songId) {
        if (_ownerOf(songId) == address(0)) revert SongNotFound(songId);
        if (_songs[songId].creator != msg.sender) revert NotSongCreator(songId, msg.sender);
        _;
    }

    modifier onlyAuthorizedConsumer() {
        if (!_authorizedConsumers[msg.sender] && msg.sender != _admin) revert NotAuthorized(msg.sender);
        _;
    }

    constructor(address adminAddress) ERC721("Rawchunks Song", "RCSONG") {
        if (adminAddress == address(0)) revert InvalidAddress();
        _admin = adminAddress;
        _nextSongId = 1;
    }

    /// @inheritdoc IFactory
    function mintSong(
        uint256 licenseAmount,
        uint256 licensePrice,
        uint256 streamPrice,
        bool remixable,
        string calldata metadataUri
    ) external returns (uint256 songId) {
        if (bytes(metadataUri).length == 0) revert EmptyMetadataUri();

        songId = _nextSongId++;
        _totalSongs++;

        _songs[songId] = Song({
            licenseAmount: licenseAmount,
            licensePrice: licensePrice,
            streamPrice: streamPrice,
            creator: msg.sender,
            remixable: remixable,
            isRemix: false,
            parentSongId: 0,
            metadataUri: metadataUri
        });

        emit SongMinted(songId, msg.sender, licenseAmount, licensePrice, streamPrice, remixable, metadataUri);
        _safeMint(msg.sender, songId);
    }

    /// @inheritdoc IFactory
    function mintRemix(
        uint256 licenseAmount,
        uint256 licensePrice,
        uint256 streamPrice,
        uint256 parentSongId,
        address remixer,
        string calldata metadataUri
    ) external onlyAuthorizedConsumer returns (uint256 songId) {
        if (bytes(metadataUri).length == 0) revert EmptyMetadataUri();
        if (remixer == address(0)) revert InvalidAddress();
        if (_ownerOf(parentSongId) == address(0)) revert SongNotFound(parentSongId);

        songId = _nextSongId++;
        _totalSongs++;

        _songs[songId] = Song({
            licenseAmount: licenseAmount,
            licensePrice: licensePrice,
            streamPrice: streamPrice,
            creator: remixer,
            remixable: false, // Remixes cannot be remixed further
            isRemix: true,
            parentSongId: parentSongId,
            metadataUri: metadataUri
        });

        emit SongMinted(songId, remixer, licenseAmount, licensePrice, streamPrice, false, metadataUri);
        _safeMint(remixer, songId);
    }

    /// @inheritdoc IFactory
    function consumeLicense(
        uint256 songId,
        address licensee
    ) external onlyAuthorizedConsumer onlyExistingSong(songId) returns (uint256 remainingLicenses) {
        if (licensee == address(0)) revert InvalidAddress();

        Song storage song = _songs[songId];
        if (!song.remixable) revert SongNotRemixable(songId);
        if (song.isRemix) revert RemixCannotBeRemixed(songId);
        if (song.licenseAmount == 0) revert NoLicensesAvailable(songId);

        song.licenseAmount--;
        _hasLicense[songId][licensee] = true;

        emit LicenseConsumed(songId, licensee, song.licenseAmount);
        return song.licenseAmount;
    }

    /// @inheritdoc IFactory
    function hasLicense(uint256 songId, address user) external view returns (bool) {
        return _hasLicense[songId][user];
    }

    /// @inheritdoc IFactory
    function canRemix(uint256 songId) external view returns (bool) {
        if (_ownerOf(songId) == address(0)) return false;
        Song storage song = _songs[songId];
        return song.remixable && !song.isRemix && song.licenseAmount > 0;
    }

    /// @inheritdoc IFactory
    function getSong(uint256 songId) external view onlyExistingSong(songId) returns (Song memory) {
        return _songs[songId];
    }

    /// @inheritdoc IFactory
    function getLicenseAmount(uint256 songId) external view onlyExistingSong(songId) returns (uint256) {
        return _songs[songId].licenseAmount;
    }

    /// @inheritdoc IFactory
    function getSongCreator(uint256 songId) external view onlyExistingSong(songId) returns (address) {
        return _songs[songId].creator;
    }

    /// @inheritdoc IFactory
    function updatePricing(
        uint256 songId,
        uint256 newLicensePrice,
        uint256 newStreamPrice
    ) external onlySongCreator(songId) {
        _songs[songId].licensePrice = newLicensePrice;
        _songs[songId].streamPrice = newStreamPrice;
        emit PricingUpdated(songId, newLicensePrice, newStreamPrice);
    }

    /// @inheritdoc IFactory
    function updateMetadataUri(uint256 songId, string calldata newMetadataUri) external onlySongCreator(songId) {
        if (bytes(newMetadataUri).length == 0) revert EmptyMetadataUri();
        _songs[songId].metadataUri = newMetadataUri;
        emit MetadataUriUpdated(songId, newMetadataUri);
    }

    /// @inheritdoc IFactory
    function songExists(uint256 songId) external view returns (bool) {
        return _ownerOf(songId) != address(0);
    }

    /// @inheritdoc IFactory
    function totalSongs() external view returns (uint256) {
        return _totalSongs;
    }

    /// @notice ERC721 token URI returning Pinata IPFS metadata URI
    function tokenURI(uint256 tokenId) public view override onlyExistingSong(tokenId) returns (string memory) {
        return _songs[tokenId].metadataUri;
    }
}
