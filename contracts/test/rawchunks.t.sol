// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Factory} from "../src/factory.sol";
import {Royalties} from "../src/royalties.sol";
import {Remix} from "../src/remix.sol";
import {IFactory} from "../src/interfaces/Ifactory.sol";
import {IRoyalties} from "../src/interfaces/Iroyalties.sol";
import {IRemix} from "../src/interfaces/Iremix.sol";

contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {}

    function decimals() public pure override returns (uint8) {
        return 6;
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract RawchunksTest is Test {
    Factory public factory;
    Royalties public royalties;
    Remix public remix;
    MockUSDC public usdc;

    address public admin = address(0xAD);
    address public treasury = address(0x78);
    address public creator = address(0xC1);
    address public remixer1 = address(0xB1);
    address public remixer2 = address(0xB2);
    address public remixer3 = address(0xB3);

    function setUp() public {
        vm.startPrank(admin);

        factory = new Factory(admin);
        royalties = new Royalties(treasury, admin);
        remix = new Remix(address(factory), address(royalties), admin);

        usdc = new MockUSDC();
        royalties.setUsdc(address(usdc));
        factory.authorizeConsumer(address(remix));

        vm.stopPrank();

        usdc.mint(remixer1, 1000 * 1e6);
        usdc.mint(remixer2, 1000 * 1e6);
        usdc.mint(remixer3, 1000 * 1e6);
    }

    function testMintSong() public {
        vm.startPrank(creator);
        uint256 songId = factory.mintSong(
            10,
            25 * 1e6, // $25 USDC
            20000,    // $0.02 USDC stream
            true,
            "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        );
        vm.stopPrank();

        assertEq(songId, 1);
        assertEq(factory.ownerOf(1), creator);
        assertEq(factory.totalSongs(), 1);
        assertEq(factory.getLicenseAmount(1), 10);
        assertEq(factory.getSongCreator(1), creator);
        assertTrue(factory.canRemix(1));

        IFactory.Song memory song = factory.getSong(1);
        assertEq(song.licenseAmount, 10);
        assertEq(song.licensePrice, 25 * 1e6);
        assertEq(song.streamPrice, 20000);
        assertEq(song.creator, creator);
        assertTrue(song.remixable);
        assertFalse(song.isRemix);
        assertEq(song.parentSongId, 0);
        assertEq(song.metadataUri, "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi");
        assertEq(factory.tokenURI(1), "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi");
    }
}
