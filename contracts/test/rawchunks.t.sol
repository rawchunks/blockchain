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

    function testLicensePurchaseAndRemix() public {
        // Creator mints song with 2 licenses at $50 USDC each
        vm.prank(creator);
        uint256 songId = factory.mintSong(2, 50 * 1e6, 20000, true, "ipfs://original-song");

        // Remixer1 approves Remix contract to spend USDC
        vm.startPrank(remixer1);
        usdc.approve(address(remix), 50 * 1e6);
        remix.buyLicense(songId);
        assertEq(factory.totalSongs(), 1);
        assertEq(remix.totalRemixes(), 0);

        IRemix.RemixParams memory params = IRemix.RemixParams({
            licenseAmount: 5,
            licensePrice: 30 * 1e6,
            streamPrice: 15000
        });

        uint256 remixId = remix.remixSong(songId, params, "ipfs://remix-song-1");
        vm.stopPrank();

        assertEq(remixId, 2);
        assertEq(factory.ownerOf(2), remixer1);
        assertEq(factory.getLicenseAmount(songId), 1); // 2 - 1 = 1 left
        assertTrue(factory.hasLicense(songId, remixer1));

        // 80% to creator ($40), 20% to treasury ($10)
        assertEq(usdc.balanceOf(creator), 40 * 1e6);
        assertEq(usdc.balanceOf(treasury), 10 * 1e6);
        assertEq(usdc.balanceOf(remixer1), (1000 - 50) * 1e6);

        // Remix properties
        IFactory.Song memory remixedSong = factory.getSong(2);
        assertTrue(remixedSong.isRemix);
        assertFalse(remixedSong.remixable);
        assertEq(remixedSong.parentSongId, songId);
        assertFalse(factory.canRemix(2));

        // Remixer 2 buys last license
        vm.startPrank(remixer2);
        usdc.approve(address(remix), 50 * 1e6);
        remix.buyLicense(songId);
        remix.remixSong(songId, params, "ipfs://remix-song-2");
        vm.stopPrank();

        assertEq(factory.getLicenseAmount(songId), 0); // 0 left
        assertFalse(factory.canRemix(songId));
        assertEq(usdc.balanceOf(creator), 80 * 1e6);
        assertEq(usdc.balanceOf(treasury), 20 * 1e6);

        // Remixer 3 tries to remix when 0 licenses left -> MUST FAIL
        vm.startPrank(remixer3);
        usdc.approve(address(remix), 50 * 1e6);
        vm.expectRevert(abi.encodeWithSelector(IFactory.NoLicensesAvailable.selector, songId));
        remix.buyLicense(songId);
        vm.stopPrank();

        vm.prank(remixer1);
        uint256 variation = remix.remixSong(songId, params, "ipfs://remix-song-1-variation");
        assertEq(variation, 4);
        assertEq(factory.getSong(variation).parentSongId, songId);
        assertEq(factory.getLicenseAmount(songId), 0);
        assertEq(usdc.balanceOf(remixer1), 950 * 1e6);
        assertEq(usdc.balanceOf(creator), 80 * 1e6);
        assertEq(usdc.balanceOf(treasury), 20 * 1e6);
        assertEq(remix.getRemixCount(songId), 3);
        uint256[] memory ids = remix.getRemixesOf(songId);
        assertEq(ids[0], 2);
        assertEq(ids[1], 3);
        assertEq(ids[2], 4);
    }

    function testCannotRemixNonRemixableSong() public {
        vm.prank(creator);
        uint256 songId = factory.mintSong(10, 50 * 1e6, 20000, false, "ipfs://non-remixable");

        vm.startPrank(remixer1);
        usdc.approve(address(remix), 50 * 1e6);
        IRemix.RemixParams memory params = IRemix.RemixParams(5, 30 * 1e6, 15000);

        vm.expectRevert(abi.encodeWithSelector(IFactory.SongNotRemixable.selector, songId));
        remix.remixSong(songId, params, "ipfs://remix");
        vm.stopPrank();
    }

    function testCannotRemixARemix() public {
        vm.prank(creator);
        uint256 songId = factory.mintSong(2, 50 * 1e6, 20000, true, "ipfs://original");

        vm.startPrank(remixer1);
        usdc.approve(address(remix), 50 * 1e6);
        IRemix.RemixParams memory params = IRemix.RemixParams(5, 30 * 1e6, 15000);
        remix.buyLicense(songId);
        uint256 remixId = remix.remixSong(songId, params, "ipfs://remix-1");
        vm.stopPrank();

        vm.startPrank(remixer2);
        usdc.approve(address(remix), 30 * 1e6);
        vm.expectRevert(abi.encodeWithSelector(IFactory.SongNotRemixable.selector, remixId));
        remix.remixSong(remixId, params, "ipfs://remix-2");
        vm.stopPrank();
    }

    function testFreeRemix() public {
        vm.prank(creator);
        uint256 songId = factory.mintSong(1, 0, 20000, true, "ipfs://free-song");

        vm.startPrank(remixer1);
        IRemix.RemixParams memory params = IRemix.RemixParams(1, 0, 20000);
        remix.buyLicense(songId);
        assertEq(factory.totalSongs(), 1);
        uint256 remixId = remix.remixSong(songId, params, "ipfs://free-remix");
        vm.stopPrank();

        assertEq(remixId, 2);
        assertEq(factory.getLicenseAmount(songId), 0);
    }
}
