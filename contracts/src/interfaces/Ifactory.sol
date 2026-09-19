// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

/**
 * @title IFactory
 * @notice Interface for Rawchunks Song Factory
 * @dev Tracks song licenses, pricing in USDC, and Pinata IPFS metadata URI
 */
interface IFactory is IERC721 {
    /// @notice Song structure stored on-chain
    struct Song {
        uint256 licenseAmount; // Remaining licenses available to be bought/consumed
        uint256 licensePrice;  // Price per license in USDC (6 decimals)
        uint256 streamPrice;   // Price per minute:stream in USDC (6 decimals)
        address creator;       // Original creator address
        bool remixable;        // Whether original can be remixed
        bool isRemix;          // True if this is a remixed variation
        uint256 parentSongId;  // Reference to original song ID (0 if original)
        string metadataUri;    // IPFS URI to Pinata metadata JSON
    }

    /// @notice Emitted when a new song is minted
    event SongMinted(
        uint256 indexed songId,
        address indexed creator,
        uint256 licenseAmount,
        uint256 licensePrice,
        uint256 streamPrice,
        bool remixable,
        string metadataUri
    );

    /// @notice Emitted when a license is consumed
    event LicenseConsumed(
        uint256 indexed songId,
        address indexed licensee,
        uint256 remainingLicenses
    );

    /// @notice Emitted when pricing is updated
    event PricingUpdated(uint256 indexed songId, uint256 licensePrice, uint256 streamPrice);

    /// @notice Emitted when metadata URI is updated
    event MetadataUriUpdated(uint256 indexed songId, string newMetadataUri);

    /// @notice Emitted when an authorized consumer is added or removed
    event ConsumerAuthorized(address indexed consumer);
    event ConsumerRevoked(address indexed consumer);

    /// @notice Emitted when admin role is transferred
    event AdminTransferred(address indexed previousAdmin, address indexed newAdmin);

    error SongNotFound(uint256 songId);
    error NotSongCreator(uint256 songId, address caller);
    error NotAuthorized(address caller);
    error SongNotRemixable(uint256 songId);
    error RemixCannotBeRemixed(uint256 songId);
    error NoLicensesAvailable(uint256 songId);
    error InvalidAddress();
    error EmptyMetadataUri();

    /**
     * @notice Mint a new song
     * @param licenseAmount Total license amount available for this song
     * @param licensePrice Price per license in USDC (6 decimals)
     * @param streamPrice Price per stream in USDC (6 decimals)
     * @param remixable Whether this song can be remixed
     * @param metadataUri IPFS URI for Pinata metadata JSON
     * @return songId Newly minted song token ID
     */
    function mintSong(
        uint256 licenseAmount,
        uint256 licensePrice,
        uint256 streamPrice,
        bool remixable,
        string calldata metadataUri
    ) external returns (uint256 songId);

    /**
     * @notice Mint a remixed song (authorized consumers only, e.g. Remix contract)
     * @param licenseAmount Total license amount allocated for the remix
     * @param licensePrice Price per license for the remix in USDC (6 decimals)
     * @param streamPrice Price per stream for the remix in USDC (6 decimals)
     * @param parentSongId Original song ID being remixed
     * @param remixer Address of the remix creator
     * @param metadataUri IPFS URI for Pinata metadata JSON
     * @return songId Newly minted remix song token ID
     */
    function mintRemix(
        uint256 licenseAmount,
        uint256 licensePrice,
        uint256 streamPrice,
        uint256 parentSongId,
        address remixer,
        string calldata metadataUri
    ) external returns (uint256 songId);

    /**
     * @notice Consume 1 unit of license amount when a license is bought
     * @param songId Song identifier
     * @param licensee Address receiving the license
     * @return remainingLicenses Number of licenses remaining
     */
    function consumeLicense(uint256 songId, address licensee) external returns (uint256 remainingLicenses);

    /**
     * @notice Check whether a user has bought a license for a song
     * @param songId Song identifier
     * @param user User address
     * @return licensed True if licensed
     */
    function hasLicense(uint256 songId, address user) external view returns (bool licensed);

    /**
     * @notice Check if a song can be remixed (remixable, not remix, licenseAmount > 0)
     * @param songId Song identifier
     * @return canRemixSong True if song can be remixed
     */
    function canRemix(uint256 songId) external view returns (bool canRemixSong);

    /**
     * @notice Get song data
     * @param songId Song identifier
     * @return song Song struct
     */
    function getSong(uint256 songId) external view returns (Song memory song);

    /**
     * @notice Get remaining license amount for a song
     * @param songId Song identifier
     * @return licenseAmount Number of licenses remaining
     */
    function getLicenseAmount(uint256 songId) external view returns (uint256 licenseAmount);

    /**
     * @notice Get song creator
     * @param songId Song identifier
     * @return creator Address of song creator
     */
    function getSongCreator(uint256 songId) external view returns (address creator);

    /**
     * @notice Check if song exists
     * @param songId Song identifier
     * @return exists True if song exists
     */
    function songExists(uint256 songId) external view returns (bool exists);

    /**
     * @notice Get total songs minted
     * @return total Total count
     */
    function totalSongs() external view returns (uint256 total);

    /**
     * @notice Update song pricing
     * @param songId Song identifier
     * @param newLicensePrice New license price in USDC (6 decimals)
     * @param newStreamPrice New stream price in USDC (6 decimals)
     */
    function updatePricing(uint256 songId, uint256 newLicensePrice, uint256 newStreamPrice) external;

    /**
     * @notice Update song metadata URI
     * @param songId Song identifier
     * @param newMetadataUri New IPFS metadata URI
     */
    function updateMetadataUri(uint256 songId, string calldata newMetadataUri) external;

    /**
     * @notice Authorize consumer contract (e.g. Remix)
     * @param consumer Consumer address
     */
    function authorizeConsumer(address consumer) external;

    /**
     * @notice Revoke consumer authorization
     * @param consumer Consumer address
     */
    function revokeConsumer(address consumer) external;

    /**
     * @notice Check if address is authorized consumer
     * @param consumer Consumer address
     * @return authorized True if authorized
     */
    function isAuthorizedConsumer(address consumer) external view returns (bool authorized);

    /**
     * @notice Transfer admin role
     * @param newAdmin Address of new admin
     */
    function transferAdmin(address newAdmin) external;

    /**
     * @notice Get admin address
     * @return admin Address of admin
     */
    function getAdmin() external view returns (address admin);
}
