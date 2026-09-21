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
}
