// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Script, console2} from "forge-std/Script.sol";
import {Factory} from "../src/factory.sol";
import {Royalties} from "../src/royalties.sol";
import {Remix} from "../src/remix.sol";

contract Rawchunks is Script {
    // Canonical CREATE2 deployer factory (deployed on all EVM chains at this exact address)
    address public constant DETERMINISTIC_CREATE2_FACTORY = 0x4e59b44847b379578588920cA78FbF26c0B4956C;

    // Salt used for deterministic CREATE2 deployment across all chains
    bytes32 public constant FACTORY_SALT = keccak256("rawchunks.v1.factory.salt");
    bytes32 public constant ROYALTIES_SALT = keccak256("rawchunks.v1.royalties.salt");
    bytes32 public constant REMIX_SALT = keccak256("rawchunks.v1.remix.salt");

    // USDC addresses per network
    address public constant BASE_SEPOLIA_USDC = 0x036CbD53842c5426634e7929541eC2318f3dCF7e;
    address public constant TEMPO_MODERATO_OUSD = 0x20c0000000000000000000006a37DA5C996874BE;
    // address public constant TEMPO_MAINNET_USDC =
    // address public constant BASE_MAINNET_USDC = 

    function computeAddress(bytes32 salt, bytes memory initCode) public pure returns (address) {
        return address(
            uint160(
                uint256(
                    keccak256(
                        abi.encodePacked(
                            bytes1(0xff),
                            DETERMINISTIC_CREATE2_FACTORY,
                            salt,
                            keccak256(initCode)
                        )
                    )
                )
            )
        );
    }

    function deployWithCreate2(bytes32 salt, bytes memory initCode) internal returns (address deployed) {
        deployed = computeAddress(salt, initCode);
        if (deployed.code.length > 0) {
            console2.log("Contract already deployed at:", deployed);
            return deployed;
        }

        bytes memory payload = abi.encodePacked(salt, initCode);
        (bool success, ) = DETERMINISTIC_CREATE2_FACTORY.call(payload);
        require(success, "CREATE2 deployment failed");
        require(deployed.code.length > 0, "Failed to verify contract code at deployed address");
    }

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
        address treasuryAddress = vm.envAddress("TREASURY_ADDRESS");
        address deployer = vm.addr(deployerPrivateKey);

        console2.log("=== Rawchunks Deterministic Deployment ===");
        console2.log("Deployer:", deployer);
        console2.log("Treasury:", treasuryAddress);
        console2.log("Chain ID:", block.chainid);

        bytes memory factoryInitCode = abi.encodePacked(type(Factory).creationCode, abi.encode(deployer));
        address factoryAddress = computeAddress(FACTORY_SALT, factoryInitCode);

        bytes memory royaltiesInitCode = abi.encodePacked(type(Royalties).creationCode, abi.encode(treasuryAddress, deployer));
        address royaltiesAddress = computeAddress(ROYALTIES_SALT, royaltiesInitCode);

        bytes memory remixInitCode = abi.encodePacked(type(Remix).creationCode, abi.encode(factoryAddress, royaltiesAddress, deployer));
        address remixAddress = computeAddress(REMIX_SALT, remixInitCode);

        console2.log("Calculated Deterministic Factory:", factoryAddress);
        console2.log("Calculated Deterministic Royalties:", royaltiesAddress);
        console2.log("Calculated Deterministic Remix:", remixAddress);

        vm.startBroadcast(deployerPrivateKey);

        address deployedFactory = deployWithCreate2(FACTORY_SALT, factoryInitCode);
        address deployedRoyalties = deployWithCreate2(ROYALTIES_SALT, royaltiesInitCode);
        address deployedRemix = deployWithCreate2(REMIX_SALT, remixInitCode);

        Factory factory = Factory(deployedFactory);
        Royalties royalties = Royalties(deployedRoyalties);

        // Configure network-specific USDC
        address chainUsdc;
        if (block.chainid == 84532) {
            chainUsdc = BASE_SEPOLIA_USDC;
        } else if (block.chainid == 42431) {
            chainUsdc = TEMPO_MODERATO_OUSD;
        } else {
            console2.log("Warning: Unknown chain ID, USDC not configured");
        }

        if (chainUsdc != address(0) && royalties.usdc() != chainUsdc) {
            royalties.setUsdc(chainUsdc);
            console2.log("Configured USDC on Royalties:", chainUsdc);
        }

        // Authorize Remix contract as consumer on Factory
        if (!factory.isAuthorizedConsumer(deployedRemix)) {
            factory.authorizeConsumer(deployedRemix);
            console2.log("Authorized Remix on Factory");
        }

        vm.stopBroadcast();

        console2.log("Deployment and configuration completed successfully.");
    }
}
