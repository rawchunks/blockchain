// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/**
 * @title IRoyalties
 * @notice Interface for Rawchunks Royalties contract
 * @dev Distributes license payments in USDC (80% creator, 20% treasury)
 */
interface IRoyalties {
    /// @notice Emitted when a license payment is distributed
    event LicensePaymentDistributed(
        uint256 indexed songId,
        address indexed payer,
        address indexed creator,
        uint256 licensePrice,
        uint256 creatorShare,
        uint256 treasuryFee
    );

    /// @notice Emitted when USDC address is updated
    event UsdcUpdated(address indexed previousUsdc, address indexed newUsdc);

    /// @notice Emitted when treasury address is updated
    event TreasuryUpdated(address indexed previousTreasury, address indexed newTreasury);

    /// @notice Emitted when admin role is transferred
    event AdminTransferred(address indexed previousAdmin, address indexed newAdmin);

    error InvalidAddress();
    error NotAdmin(address caller);
    error UsdcNotSet();

    /**
     * @notice Distribute license payment in USDC (80% creator, 20% treasury)
     * @param songId Song identifier
     * @param payer Address paying the license
     * @param creator Original song creator
     * @param licensePrice License price in USDC (6 decimals)
     * @return creatorShare Amount sent to creator
     * @return treasuryFee Amount sent to treasury
     */
    function distributeLicensePayment(
        uint256 songId,
        address payer,
        address creator,
        uint256 licensePrice
    ) external returns (uint256 creatorShare, uint256 treasuryFee);

    /**
     * @notice Calculate 80/20 split for a given license price
     * @param licensePrice License price in USDC (6 decimals)
     * @return creatorShare 80% to creator
     * @return treasuryFee 20% to treasury
     */
    function calculateSplit(uint256 licensePrice) external pure returns (uint256 creatorShare, uint256 treasuryFee);

    /**
     * @notice Set USDC contract address
     * @param newUsdc Address of USDC ERC20
     */
    function setUsdc(address newUsdc) external;

    /**
     * @notice Set treasury address
     * @param newTreasury Address of treasury
     */
    function setTreasury(address newTreasury) external;

    /**
     * @notice Get current USDC token address
     * @return usdcAddress USDC contract address
     */
    function usdc() external view returns (address usdcAddress);

    /**
     * @notice Get current treasury address
     * @return treasuryAddress Treasury address
     */
    function treasury() external view returns (address treasuryAddress);

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
