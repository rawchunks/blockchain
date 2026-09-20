// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/**
 * @title IRemix
 * @notice Interface for Rawchunks Remix contract
 * @dev Separates ongoing license rights from variation minting
 */
interface IRemix {
    event LicensePurchased(uint256 indexed originalSongId, address indexed licensee, uint256 price);
    /// @notice Emitted when a song is remixed
    event SongRemixed(
        uint256 indexed originalSongId,
        uint256 indexed remixedSongId,
        address indexed remixer,
        string metadataUri
    );

    error SongNotFound(uint256 songId);
    error SongNotRemixable(uint256 songId);
    error RemixCannotBeRemixed(uint256 songId);
    error NoLicensesAvailable(uint256 songId);
    error LicenseRequired(uint256 songId, address licensee);

    /// @notice Purchase ongoing rights to an original song without minting a variation
    function buyLicense(uint256 originalSongId) external;

    /**
     * @notice Parameters for the remixed song
     * @param licenseAmount Total license amount for the remix
     * @param licensePrice License price for the remix in USDC (6 decimals)
     * @param streamPrice Stream price for the remix in USDC (6 decimals)
     */
    struct RemixParams {
        uint256 licenseAmount;
        uint256 licensePrice;
        uint256 streamPrice;
    }

    /**
     * @notice Mint a variation of a song for which the caller already owns a license
     * @param originalSongId The original song to remix
     * @param params Parameters for the remixed variation
     * @param metadataUri IPFS URI for the remixed song's Pinata metadata
     * @return remixedSongId The new remix song ID
     */
    function remixSong(
        uint256 originalSongId,
        RemixParams calldata params,
        string calldata metadataUri
    ) external returns (uint256 remixedSongId);

    /**
     * @notice Check if a song can be remixed
     * @param songId Song identifier
     * @return canRemix True if song can be remixed
     */
    function canRemix(uint256 songId) external view returns (bool canRemix);

    /**
     * @notice Get all remix IDs created from an original song
     * @param originalSongId Original song ID
     * @return remixIds Array of remix song IDs
     */
    function getRemixesOf(uint256 originalSongId) external view returns (uint256[] memory remixIds);

    /**
     * @notice Get remix count for an original song
     * @param originalSongId Original song ID
     * @return count Number of remixes created
     */
    function getRemixCount(uint256 originalSongId) external view returns (uint256 count);

    /**
     * @notice Total remixes created across all songs
     * @return total Total count
     */
    function totalRemixes() external view returns (uint256 total);

    /**
     * @notice Get Factory contract address
     * @return factoryAddress Factory address
     */
    function factory() external view returns (address factoryAddress);

    /**
     * @notice Get Royalties contract address
     * @return royaltiesAddress Royalties address
     */
    function royalties() external view returns (address royaltiesAddress);
}
