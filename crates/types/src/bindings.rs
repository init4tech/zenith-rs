use alloy_sol_types::sol;

sol!(
    #[sol(rpc)]
    ZenithContract,
    r#"[
        {
            "type": "constructor",
            "inputs": [
            {
                "name": "defaultRollupChainId",
                "type": "uint256",
                "internalType": "uint256"
            },
            {
                "name": "admin",
                "type": "address",
                "internalType": "address"
            }
            ],
            "stateMutability": "nonpayable"
        },
        {
            "type": "fallback",
            "stateMutability": "payable"
        },
        {
            "type": "receive",
            "stateMutability": "payable"
        },
        {
            "type": "function",
            "name": "DEFAULT_ADMIN_ROLE",
            "inputs": [],
            "outputs": [
            {
                "name": "",
                "type": "bytes32",
                "internalType": "bytes32"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "SEQUENCER_ROLE",
            "inputs": [],
            "outputs": [
            {
                "name": "",
                "type": "bytes32",
                "internalType": "bytes32"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "acceptDefaultAdminTransfer",
            "inputs": [],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "beginDefaultAdminTransfer",
            "inputs": [
            {
                "name": "newAdmin",
                "type": "address",
                "internalType": "address"
            }
            ],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "blockCommitment",
            "inputs": [
            {
                "name": "header",
                "type": "tuple",
                "internalType": "struct Zenith.BlockHeader",
                "components": [
                {
                    "name": "rollupChainId",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "sequence",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "confirmBy",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "gasLimit",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "rewardAddress",
                    "type": "address",
                    "internalType": "address"
                }
                ]
            },
            {
                "name": "blockDataHash",
                "type": "bytes32",
                "internalType": "bytes32"
            }
            ],
            "outputs": [
            {
                "name": "commit",
                "type": "bytes32",
                "internalType": "bytes32"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "cancelDefaultAdminTransfer",
            "inputs": [],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "changeDefaultAdminDelay",
            "inputs": [
            {
                "name": "newDelay",
                "type": "uint48",
                "internalType": "uint48"
            }
            ],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "defaultAdmin",
            "inputs": [],
            "outputs": [
            {
                "name": "",
                "type": "address",
                "internalType": "address"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "defaultAdminDelay",
            "inputs": [],
            "outputs": [
            {
                "name": "",
                "type": "uint48",
                "internalType": "uint48"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "defaultAdminDelayIncreaseWait",
            "inputs": [],
            "outputs": [
            {
                "name": "",
                "type": "uint48",
                "internalType": "uint48"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "enter",
            "inputs": [
            {
                "name": "rollupChainId",
                "type": "uint256",
                "internalType": "uint256"
            },
            {
                "name": "rollupRecipient",
                "type": "address",
                "internalType": "address"
            },
            {
                "name": "token",
                "type": "address",
                "internalType": "address"
            },
            {
                "name": "amount",
                "type": "uint256",
                "internalType": "uint256"
            }
            ],
            "outputs": [],
            "stateMutability": "payable"
        },
        {
            "type": "function",
            "name": "enter",
            "inputs": [
            {
                "name": "rollupChainId",
                "type": "uint256",
                "internalType": "uint256"
            },
            {
                "name": "rollupRecipient",
                "type": "address",
                "internalType": "address"
            }
            ],
            "outputs": [],
            "stateMutability": "payable"
        },
        {
            "type": "function",
            "name": "fulfillExits",
            "inputs": [
            {
                "name": "orders",
                "type": "tuple[]",
                "internalType": "struct Passage.ExitOrder[]",
                "components": [
                {
                    "name": "rollupChainId",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "token",
                    "type": "address",
                    "internalType": "address"
                },
                {
                    "name": "recipient",
                    "type": "address",
                    "internalType": "address"
                },
                {
                    "name": "amount",
                    "type": "uint256",
                    "internalType": "uint256"
                }
                ]
            }
            ],
            "outputs": [],
            "stateMutability": "payable"
        },
        {
            "type": "function",
            "name": "getRoleAdmin",
            "inputs": [
            {
                "name": "role",
                "type": "bytes32",
                "internalType": "bytes32"
            }
            ],
            "outputs": [
            {
                "name": "",
                "type": "bytes32",
                "internalType": "bytes32"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "grantRole",
            "inputs": [
            {
                "name": "role",
                "type": "bytes32",
                "internalType": "bytes32"
            },
            {
                "name": "account",
                "type": "address",
                "internalType": "address"
            }
            ],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "hasRole",
            "inputs": [
            {
                "name": "role",
                "type": "bytes32",
                "internalType": "bytes32"
            },
            {
                "name": "account",
                "type": "address",
                "internalType": "address"
            }
            ],
            "outputs": [
            {
                "name": "",
                "type": "bool",
                "internalType": "bool"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "lastSubmittedAtBlock",
            "inputs": [
            {
                "name": "",
                "type": "uint256",
                "internalType": "uint256"
            }
            ],
            "outputs": [
            {
                "name": "",
                "type": "uint256",
                "internalType": "uint256"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "nextSequence",
            "inputs": [
            {
                "name": "",
                "type": "uint256",
                "internalType": "uint256"
            }
            ],
            "outputs": [
            {
                "name": "",
                "type": "uint256",
                "internalType": "uint256"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "owner",
            "inputs": [],
            "outputs": [
            {
                "name": "",
                "type": "address",
                "internalType": "address"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "pendingDefaultAdmin",
            "inputs": [],
            "outputs": [
            {
                "name": "newAdmin",
                "type": "address",
                "internalType": "address"
            },
            {
                "name": "schedule",
                "type": "uint48",
                "internalType": "uint48"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "pendingDefaultAdminDelay",
            "inputs": [],
            "outputs": [
            {
                "name": "newDelay",
                "type": "uint48",
                "internalType": "uint48"
            },
            {
                "name": "schedule",
                "type": "uint48",
                "internalType": "uint48"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "renounceRole",
            "inputs": [
            {
                "name": "role",
                "type": "bytes32",
                "internalType": "bytes32"
            },
            {
                "name": "account",
                "type": "address",
                "internalType": "address"
            }
            ],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "revokeRole",
            "inputs": [
            {
                "name": "role",
                "type": "bytes32",
                "internalType": "bytes32"
            },
            {
                "name": "account",
                "type": "address",
                "internalType": "address"
            }
            ],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "rollbackDefaultAdminDelay",
            "inputs": [],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "submitBlock",
            "inputs": [
            {
                "name": "header",
                "type": "tuple",
                "internalType": "struct Zenith.BlockHeader",
                "components": [
                {
                    "name": "rollupChainId",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "sequence",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "confirmBy",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "gasLimit",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "rewardAddress",
                    "type": "address",
                    "internalType": "address"
                }
                ]
            },
            {
                "name": "blockDataHash",
                "type": "bytes32",
                "internalType": "bytes32"
            },
            {
                "name": "v",
                "type": "uint8",
                "internalType": "uint8"
            },
            {
                "name": "r",
                "type": "bytes32",
                "internalType": "bytes32"
            },
            {
                "name": "s",
                "type": "bytes32",
                "internalType": "bytes32"
            },
            {
                "name": "blockData",
                "type": "bytes",
                "internalType": "bytes"
            }
            ],
            "outputs": [],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "supportsInterface",
            "inputs": [
            {
                "name": "interfaceId",
                "type": "bytes4",
                "internalType": "bytes4"
            }
            ],
            "outputs": [
            {
                "name": "",
                "type": "bool",
                "internalType": "bool"
            }
            ],
            "stateMutability": "view"
        },
        {
            "type": "event",
            "name": "BlockData",
            "inputs": [
            {
                "name": "blockData",
                "type": "bytes",
                "indexed": false,
                "internalType": "bytes"
            }
            ],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "BlockSubmitted",
            "inputs": [
            {
                "name": "sequencer",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            },
            {
                "name": "header",
                "type": "tuple",
                "indexed": true,
                "internalType": "struct Zenith.BlockHeader",
                "components": [
                {
                    "name": "rollupChainId",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "sequence",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "confirmBy",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "gasLimit",
                    "type": "uint256",
                    "internalType": "uint256"
                },
                {
                    "name": "rewardAddress",
                    "type": "address",
                    "internalType": "address"
                }
                ]
            },
            {
                "name": "blockDataHash",
                "type": "bytes32",
                "indexed": false,
                "internalType": "bytes32"
            }
            ],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "DefaultAdminDelayChangeCanceled",
            "inputs": [],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "DefaultAdminDelayChangeScheduled",
            "inputs": [
            {
                "name": "newDelay",
                "type": "uint48",
                "indexed": false,
                "internalType": "uint48"
            },
            {
                "name": "effectSchedule",
                "type": "uint48",
                "indexed": false,
                "internalType": "uint48"
            }
            ],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "DefaultAdminTransferCanceled",
            "inputs": [],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "DefaultAdminTransferScheduled",
            "inputs": [
            {
                "name": "newAdmin",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            },
            {
                "name": "acceptSchedule",
                "type": "uint48",
                "indexed": false,
                "internalType": "uint48"
            }
            ],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "Enter",
            "inputs": [
            {
                "name": "rollupChainId",
                "type": "uint256",
                "indexed": false,
                "internalType": "uint256"
            },
            {
                "name": "token",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            },
            {
                "name": "rollupRecipient",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            },
            {
                "name": "amount",
                "type": "uint256",
                "indexed": false,
                "internalType": "uint256"
            }
            ],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "ExitFilled",
            "inputs": [
            {
                "name": "rollupChainId",
                "type": "uint256",
                "indexed": false,
                "internalType": "uint256"
            },
            {
                "name": "token",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            },
            {
                "name": "hostRecipient",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            },
            {
                "name": "amount",
                "type": "uint256",
                "indexed": false,
                "internalType": "uint256"
            }
            ],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "RoleAdminChanged",
            "inputs": [
            {
                "name": "role",
                "type": "bytes32",
                "indexed": true,
                "internalType": "bytes32"
            },
            {
                "name": "previousAdminRole",
                "type": "bytes32",
                "indexed": true,
                "internalType": "bytes32"
            },
            {
                "name": "newAdminRole",
                "type": "bytes32",
                "indexed": true,
                "internalType": "bytes32"
            }
            ],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "RoleGranted",
            "inputs": [
            {
                "name": "role",
                "type": "bytes32",
                "indexed": true,
                "internalType": "bytes32"
            },
            {
                "name": "account",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            },
            {
                "name": "sender",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            }
            ],
            "anonymous": false
        },
        {
            "type": "event",
            "name": "RoleRevoked",
            "inputs": [
            {
                "name": "role",
                "type": "bytes32",
                "indexed": true,
                "internalType": "bytes32"
            },
            {
                "name": "account",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            },
            {
                "name": "sender",
                "type": "address",
                "indexed": true,
                "internalType": "address"
            }
            ],
            "anonymous": false
        },
        {
            "type": "error",
            "name": "AccessControlBadConfirmation",
            "inputs": []
        },
        {
            "type": "error",
            "name": "AccessControlEnforcedDefaultAdminDelay",
            "inputs": [
            {
                "name": "schedule",
                "type": "uint48",
                "internalType": "uint48"
            }
            ]
        },
        {
            "type": "error",
            "name": "AccessControlEnforcedDefaultAdminRules",
            "inputs": []
        },
        {
            "type": "error",
            "name": "AccessControlInvalidDefaultAdmin",
            "inputs": [
            {
                "name": "defaultAdmin",
                "type": "address",
                "internalType": "address"
            }
            ]
        },
        {
            "type": "error",
            "name": "AccessControlUnauthorizedAccount",
            "inputs": [
            {
                "name": "account",
                "type": "address",
                "internalType": "address"
            },
            {
                "name": "neededRole",
                "type": "bytes32",
                "internalType": "bytes32"
            }
            ]
        },
        {
            "type": "error",
            "name": "BadSequence",
            "inputs": [
            {
                "name": "expected",
                "type": "uint256",
                "internalType": "uint256"
            }
            ]
        },
        {
            "type": "error",
            "name": "BadSignature",
            "inputs": [
            {
                "name": "derivedSequencer",
                "type": "address",
                "internalType": "address"
            }
            ]
        },
        {
            "type": "error",
            "name": "BlockExpired",
            "inputs": []
        },
        {
            "type": "error",
            "name": "OneRollupBlockPerHostBlock",
            "inputs": []
        },
        {
            "type": "error",
            "name": "OrderExpired",
            "inputs": []
        },
        {
            "type": "error",
            "name": "SafeCastOverflowedUintDowncast",
            "inputs": [
            {
                "name": "bits",
                "type": "uint8",
                "internalType": "uint8"
            },
            {
                "name": "value",
                "type": "uint256",
                "internalType": "uint256"
            }
            ]
        }
        ]"#
);
