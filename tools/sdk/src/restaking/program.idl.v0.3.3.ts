/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/restaking.json`.
 */
export type Restaking = {
    "address": "fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3",
    "metadata": {
        "name": "restaking",
        "version": "0.3.3",
        "spec": "0.1.0",
        "description": "Fragmetric Liquid Restaking Token Program"
    },
    "instructions": [
        {
            "name": "adminInitializeExtraAccountMetaList",
            "discriminator": [
                43,
                34,
                13,
                49,
                167,
                88,
                235,
                235
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "admin",
                    "signer": true,
                    "address": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint"
                },
                {
                    "name": "extraAccountMetaList",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    101,
                                    120,
                                    116,
                                    114,
                                    97,
                                    45,
                                    97,
                                    99,
                                    99,
                                    111,
                                    117,
                                    110,
                                    116,
                                    45,
                                    109,
                                    101,
                                    116,
                                    97,
                                    115
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "adminInitializeFundAccount",
            "discriminator": [
                83,
                184,
                197,
                143,
                118,
                192,
                56,
                15
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "admin",
                    "signer": true,
                    "address": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundReceiptTokenLockAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "fundReserveAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    114,
                                    101,
                                    115,
                                    101,
                                    114,
                                    118,
                                    101
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "adminInitializeNormalizedTokenPoolAccount",
            "discriminator": [
                36,
                90,
                87,
                197,
                124,
                174,
                14,
                225
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "admin",
                    "signer": true,
                    "address": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "normalizedTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "normalizedTokenMint",
                    "writable": true
                },
                {
                    "name": "normalizedTokenPoolAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    112,
                                    111,
                                    111,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "adminInitializeRewardAccount",
            "discriminator": [
                208,
                48,
                70,
                171,
                86,
                38,
                29,
                149
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "admin",
                    "signer": true,
                    "address": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint"
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "adminUpdateExtraAccountMetaListIfNeeded",
            "discriminator": [
                113,
                124,
                72,
                210,
                237,
                164,
                96,
                241
            ],
            "accounts": [
                {
                    "name": "payer",
                    "signer": true
                },
                {
                    "name": "admin",
                    "signer": true,
                    "address": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
                },
                {
                    "name": "receiptTokenMint"
                },
                {
                    "name": "extraAccountMetaList",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    101,
                                    120,
                                    116,
                                    114,
                                    97,
                                    45,
                                    97,
                                    99,
                                    99,
                                    111,
                                    117,
                                    110,
                                    116,
                                    45,
                                    109,
                                    101,
                                    116,
                                    97,
                                    115
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "adminUpdateFundAccountIfNeeded",
            "discriminator": [
                53,
                204,
                67,
                56,
                198,
                113,
                243,
                34
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "admin",
                    "signer": true,
                    "address": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint"
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundReserveAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    114,
                                    101,
                                    115,
                                    101,
                                    114,
                                    118,
                                    101
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "desiredAccountSize",
                    "type": {
                        "option": "u32"
                    }
                }
            ]
        },
        {
            "name": "adminUpdateNormalizedTokenPoolAccountIfNeeded",
            "discriminator": [
                117,
                212,
                78,
                133,
                31,
                164,
                123,
                241
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "admin",
                    "signer": true,
                    "address": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "normalizedTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "normalizedTokenMint",
                    "relations": [
                        "normalizedTokenPoolAccount"
                    ]
                },
                {
                    "name": "normalizedTokenPoolAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    112,
                                    111,
                                    111,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "adminUpdateRewardAccountIfNeeded",
            "discriminator": [
                113,
                211,
                75,
                86,
                235,
                248,
                240,
                2
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "admin",
                    "signer": true,
                    "address": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint"
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "desiredAccountSize",
                    "type": {
                        "option": "u32"
                    }
                }
            ]
        },
        {
            "name": "fundManagerAddNormalizedTokenPoolSupportedToken",
            "discriminator": [
                173,
                135,
                121,
                96,
                30,
                138,
                56,
                27
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "normalizedTokenMint",
                    "relations": [
                        "normalizedTokenPoolAccount"
                    ]
                },
                {
                    "name": "normalizedTokenPoolAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    112,
                                    111,
                                    111,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "normalizedTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "supportedTokenMint"
                },
                {
                    "name": "supportedTokenProgram"
                },
                {
                    "name": "normalizedTokenPoolSupportedTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "normalizedTokenPoolAccount"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "pricingSource",
                    "type": {
                        "defined": {
                            "name": "tokenPricingSource"
                        }
                    }
                }
            ]
        },
        {
            "name": "fundManagerAddReward",
            "discriminator": [
                26,
                6,
                104,
                77,
                57,
                237,
                13,
                5
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount"
                    ]
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardTokenMint",
                    "optional": true
                },
                {
                    "name": "rewardTokenProgram",
                    "optional": true
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "description",
                    "type": "string"
                },
                {
                    "name": "rewardType",
                    "type": {
                        "defined": {
                            "name": "rewardType"
                        }
                    }
                }
            ]
        },
        {
            "name": "fundManagerAddRewardPool",
            "discriminator": [
                222,
                241,
                120,
                225,
                5,
                76,
                175,
                136
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount"
                    ]
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "holderId",
                    "type": {
                        "option": "u8"
                    }
                },
                {
                    "name": "customContributionAccrualRateEnabled",
                    "type": "bool"
                }
            ]
        },
        {
            "name": "fundManagerAddRewardPoolHolder",
            "discriminator": [
                79,
                160,
                90,
                79,
                137,
                135,
                197,
                134
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount"
                    ]
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "description",
                    "type": "string"
                },
                {
                    "name": "pubkeys",
                    "type": {
                        "vec": "pubkey"
                    }
                }
            ]
        },
        {
            "name": "fundManagerAddSupportedToken",
            "discriminator": [
                0,
                137,
                153,
                52,
                179,
                163,
                4,
                20
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "supportedTokenMint"
                },
                {
                    "name": "supportedTokenProgram"
                },
                {
                    "name": "supportedTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "pricingSource",
                    "type": {
                        "defined": {
                            "name": "tokenPricingSource"
                        }
                    }
                }
            ]
        },
        {
            "name": "fundManagerClearUserSolWithdrawalRequests",
            "discriminator": [
                229,
                235,
                96,
                236,
                74,
                245,
                85,
                243
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "fundAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "const",
                                "value": [
                                    214,
                                    52,
                                    8,
                                    155,
                                    182,
                                    149,
                                    115,
                                    57,
                                    20,
                                    131,
                                    125,
                                    232,
                                    82,
                                    251,
                                    210,
                                    76,
                                    255,
                                    40,
                                    78,
                                    39,
                                    34,
                                    166,
                                    52,
                                    128,
                                    105,
                                    118,
                                    67,
                                    202,
                                    117,
                                    247,
                                    108,
                                    146
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "const",
                                "value": [
                                    214,
                                    52,
                                    8,
                                    155,
                                    182,
                                    149,
                                    115,
                                    57,
                                    20,
                                    131,
                                    125,
                                    232,
                                    82,
                                    251,
                                    210,
                                    76,
                                    255,
                                    40,
                                    78,
                                    39,
                                    34,
                                    166,
                                    52,
                                    128,
                                    105,
                                    118,
                                    67,
                                    202,
                                    117,
                                    247,
                                    108,
                                    146
                                ]
                            },
                            {
                                "kind": "arg",
                                "path": "user"
                            }
                        ]
                    }
                }
            ],
            "args": [
                {
                    "name": "user",
                    "type": "pubkey"
                },
                {
                    "name": "numExpectedRequestsLeft",
                    "type": "u8"
                }
            ]
        },
        {
            "name": "fundManagerCloseFundAccount",
            "discriminator": [
                158,
                192,
                72,
                180,
                218,
                61,
                228,
                156
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "const",
                                "value": [
                                    214,
                                    52,
                                    8,
                                    155,
                                    182,
                                    149,
                                    115,
                                    57,
                                    20,
                                    131,
                                    125,
                                    232,
                                    82,
                                    251,
                                    210,
                                    76,
                                    255,
                                    40,
                                    78,
                                    39,
                                    34,
                                    166,
                                    52,
                                    128,
                                    105,
                                    118,
                                    67,
                                    202,
                                    117,
                                    247,
                                    108,
                                    146
                                ]
                            }
                        ]
                    }
                }
            ],
            "args": []
        },
        {
            "name": "fundManagerCloseRewardPool",
            "discriminator": [
                159,
                24,
                238,
                47,
                253,
                39,
                6,
                30
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount"
                    ]
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "rewardPoolId",
                    "type": "u8"
                }
            ]
        },
        {
            "name": "fundManagerInitializeFundJitoRestakingVault",
            "discriminator": [
                94,
                33,
                145,
                222,
                177,
                170,
                211,
                74
            ],
            "accounts": [
                {
                    "name": "admin",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "vaultProgram",
                    "address": "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8"
                },
                {
                    "name": "vaultAccount"
                },
                {
                    "name": "vaultReceiptTokenMint"
                },
                {
                    "name": "vaultReceiptTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "vaultSupportedTokenMint"
                },
                {
                    "name": "vaultSupportedTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "fundVaultReceiptTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "vaultReceiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "vaultReceiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "fundVaultSupportedTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "vaultSupportedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "vaultSupportedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "vaultVaultSupportedTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "vaultAccount"
                            },
                            {
                                "kind": "account",
                                "path": "vaultSupportedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "vaultSupportedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "fundManagerInitializeFundNormalizedToken",
            "discriminator": [
                210,
                163,
                184,
                165,
                127,
                40,
                122,
                23
            ],
            "accounts": [
                {
                    "name": "admin",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "normalizedTokenMint",
                    "relations": [
                        "normalizedTokenPoolAccount"
                    ]
                },
                {
                    "name": "normalizedTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "fundNormalizedTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "normalizedTokenPoolAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    112,
                                    111,
                                    111,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "fundManagerSettleReward",
            "discriminator": [
                105,
                92,
                118,
                15,
                173,
                135,
                98,
                86
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount"
                    ]
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardTokenMint",
                    "optional": true
                },
                {
                    "name": "rewardTokenProgram",
                    "optional": true
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "rewardPoolId",
                    "type": "u8"
                },
                {
                    "name": "rewardId",
                    "type": "u16"
                },
                {
                    "name": "amount",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "fundManagerUpdateFundStrategy",
            "discriminator": [
                66,
                200,
                217,
                64,
                201,
                228,
                239,
                193
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "depositEnabled",
                    "type": "bool"
                },
                {
                    "name": "withdrawalEnabled",
                    "type": "bool"
                },
                {
                    "name": "withdrawalFeeRateBps",
                    "type": "u16"
                },
                {
                    "name": "withdrawalBatchThresholdSeconds",
                    "type": "i64"
                }
            ]
        },
        {
            "name": "fundManagerUpdateRestakingVaultDelegationStrategy",
            "discriminator": [
                54,
                180,
                250,
                68,
                121,
                2,
                143,
                87
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "vault",
                    "type": "pubkey"
                },
                {
                    "name": "operator",
                    "type": "pubkey"
                },
                {
                    "name": "tokenAllocationWeight",
                    "type": "u64"
                },
                {
                    "name": "tokenAllocationCapacityAmount",
                    "type": "u64"
                },
                {
                    "name": "tokenRedelegatingAmount",
                    "type": {
                        "option": "u64"
                    }
                }
            ]
        },
        {
            "name": "fundManagerUpdateRestakingVaultStrategy",
            "discriminator": [
                131,
                35,
                217,
                161,
                90,
                24,
                63,
                133
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "vault",
                    "type": "pubkey"
                },
                {
                    "name": "solAllocationWeight",
                    "type": "u64"
                },
                {
                    "name": "solAllocationCapacityAmount",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "fundManagerUpdateSolStrategy",
            "discriminator": [
                107,
                157,
                24,
                119,
                5,
                88,
                154,
                147
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "solDepositable",
                    "type": "bool"
                },
                {
                    "name": "solAccumulatedDepositCapacityAmount",
                    "type": "u64"
                },
                {
                    "name": "solAccumulatedDepositAmount",
                    "type": {
                        "option": "u64"
                    }
                },
                {
                    "name": "solWithdrawable",
                    "type": "bool"
                },
                {
                    "name": "solWithdrawalNormalReserveRateBps",
                    "type": "u16"
                },
                {
                    "name": "solWithdrawalNormalReserveMaxAmount",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "fundManagerUpdateSupportedTokenStrategy",
            "discriminator": [
                131,
                168,
                49,
                206,
                73,
                18,
                137,
                219
            ],
            "accounts": [
                {
                    "name": "fundManager",
                    "signer": true,
                    "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "tokenMint",
                    "type": "pubkey"
                },
                {
                    "name": "tokenDepositable",
                    "type": "bool"
                },
                {
                    "name": "tokenAccumulatedDepositCapacityAmount",
                    "type": "u64"
                },
                {
                    "name": "tokenAccumulatedDepositAmount",
                    "type": {
                        "option": "u64"
                    }
                },
                {
                    "name": "tokenWithdrawable",
                    "type": "bool"
                },
                {
                    "name": "tokenWithdrawalNormalReserveRateBps",
                    "type": "u16"
                },
                {
                    "name": "tokenWithdrawalNormalReserveMaxAmount",
                    "type": "u64"
                },
                {
                    "name": "tokenRebalancingAmount",
                    "type": {
                        "option": "u64"
                    }
                },
                {
                    "name": "solAllocationWeight",
                    "type": "u64"
                },
                {
                    "name": "solAllocationCapacityAmount",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "operatorDonateSolToFund",
            "discriminator": [
                88,
                167,
                224,
                32,
                221,
                203,
                157,
                69
            ],
            "accounts": [
                {
                    "name": "operator",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    114,
                                    101,
                                    115,
                                    101,
                                    114,
                                    118,
                                    101
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "amount",
                    "type": "u64"
                },
                {
                    "name": "offsetReceivable",
                    "type": "bool"
                }
            ]
        },
        {
            "name": "operatorDonateSupportedTokenToFund",
            "discriminator": [
                116,
                216,
                13,
                162,
                86,
                164,
                43,
                93
            ],
            "accounts": [
                {
                    "name": "operator",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "supportedTokenProgram"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "supportedTokenMint"
                },
                {
                    "name": "fundSupportedTokenReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "operatorSupportedTokenAccount",
                    "writable": true
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "amount",
                    "type": "u64"
                },
                {
                    "name": "offsetReceivable",
                    "type": "bool"
                }
            ]
        },
        {
            "name": "operatorLogMessage",
            "discriminator": [
                104,
                51,
                132,
                76,
                54,
                74,
                57,
                199
            ],
            "accounts": [
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "message",
                    "type": "string"
                }
            ]
        },
        {
            "name": "operatorRunFundCommand",
            "discriminator": [
                73,
                116,
                27,
                23,
                118,
                153,
                196,
                14
            ],
            "accounts": [
                {
                    "name": "operator",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "forceResetCommand",
                    "type": {
                        "option": {
                            "defined": {
                                "name": "operationCommandEntry"
                            }
                        }
                    }
                }
            ]
        },
        {
            "name": "operatorUpdateFundPrices",
            "discriminator": [
                253,
                219,
                211,
                59,
                254,
                151,
                126,
                161
            ],
            "accounts": [
                {
                    "name": "operator",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount"
                    ]
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "operatorUpdateNormalizedTokenPoolPrices",
            "discriminator": [
                59,
                127,
                178,
                59,
                73,
                213,
                181,
                204
            ],
            "accounts": [
                {
                    "name": "operator",
                    "signer": true
                },
                {
                    "name": "normalizedTokenMint",
                    "writable": true,
                    "relations": [
                        "normalizedTokenPoolAccount"
                    ]
                },
                {
                    "name": "normalizedTokenPoolAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    112,
                                    111,
                                    111,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "normalizedTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "operatorUpdateRewardPools",
            "discriminator": [
                50,
                3,
                240,
                113,
                3,
                164,
                2,
                41
            ],
            "accounts": [
                {
                    "name": "operator",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount"
                    ]
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "slasherInitializeNormalizedTokenWithdrawalAccount",
            "discriminator": [
                180,
                112,
                136,
                49,
                174,
                179,
                50,
                47
            ],
            "accounts": [
                {
                    "name": "payer",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "slasher",
                    "signer": true
                },
                {
                    "name": "normalizedTokenMint",
                    "writable": true,
                    "relations": [
                        "normalizedTokenPoolAccount"
                    ]
                },
                {
                    "name": "normalizedTokenPoolAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    112,
                                    111,
                                    111,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "normalizedTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "slasherNormalizedTokenWithdrawalTicketAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    119,
                                    105,
                                    116,
                                    104,
                                    100,
                                    114,
                                    97,
                                    119,
                                    97,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "slasher"
                            }
                        ]
                    }
                },
                {
                    "name": "slasherNormalizedTokenAccount",
                    "writable": true
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "slasherWithdrawNormalizedToken",
            "discriminator": [
                30,
                86,
                7,
                231,
                47,
                59,
                162,
                214
            ],
            "accounts": [
                {
                    "name": "slasher",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "normalizedTokenMint",
                    "writable": true,
                    "relations": [
                        "normalizedTokenPoolAccount",
                        "slasherNormalizedTokenWithdrawalTicketAccount"
                    ]
                },
                {
                    "name": "normalizedTokenPoolAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    112,
                                    111,
                                    111,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "normalizedTokenProgram",
                    "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                },
                {
                    "name": "slasherNormalizedTokenWithdrawalTicketAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    110,
                                    116,
                                    95,
                                    119,
                                    105,
                                    116,
                                    104,
                                    100,
                                    114,
                                    97,
                                    119,
                                    97,
                                    108
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "normalizedTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "slasher"
                            }
                        ]
                    }
                },
                {
                    "name": "supportedTokenMint"
                },
                {
                    "name": "supportedTokenProgram"
                },
                {
                    "name": "normalizedTokenPoolSupportedTokenReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "normalizedTokenPoolAccount"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "destinationSupportedTokenAccount",
                    "writable": true
                },
                {
                    "name": "destinationRentLamportsAccount",
                    "writable": true
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "tokenTransferHook",
            "discriminator": [
                105,
                37,
                101,
                197,
                75,
                251,
                102,
                26
            ],
            "accounts": [
                {
                    "name": "sourceReceiptTokenAccount"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount",
                        "rewardAccount"
                    ]
                },
                {
                    "name": "destinationReceiptTokenAccount"
                },
                {
                    "name": "owner"
                },
                {
                    "name": "extraAccountMetaList",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    101,
                                    120,
                                    116,
                                    114,
                                    97,
                                    45,
                                    97,
                                    99,
                                    99,
                                    111,
                                    117,
                                    110,
                                    116,
                                    45,
                                    109,
                                    101,
                                    116,
                                    97,
                                    115
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                }
            ],
            "args": [
                {
                    "name": "amount",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "userCancelWithdrawalRequest",
            "discriminator": [
                187,
                80,
                45,
                65,
                239,
                189,
                78,
                102
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userFundAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount",
                        "userFundAccount",
                        "rewardAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "receiptTokenLockAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "userReceiptTokenAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    114,
                                    101,
                                    115,
                                    101,
                                    114,
                                    118,
                                    101
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "instructionsSysvar",
                    "address": "Sysvar1nstructions1111111111111111111111111"
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "requestId",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "userClaimRewards",
            "discriminator": [
                8,
                211,
                145,
                71,
                169,
                22,
                80,
                33
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "rewardPoolId",
                    "type": "u8"
                },
                {
                    "name": "rewardId",
                    "type": "u8"
                }
            ]
        },
        {
            "name": "userDepositSol",
            "discriminator": [
                9,
                201,
                63,
                79,
                105,
                75,
                147,
                47
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userFundAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount",
                        "userFundAccount",
                        "rewardAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "receiptTokenLockAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "userReceiptTokenAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    114,
                                    101,
                                    115,
                                    101,
                                    114,
                                    118,
                                    101
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "instructionsSysvar",
                    "address": "Sysvar1nstructions1111111111111111111111111"
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "amount",
                    "type": "u64"
                },
                {
                    "name": "metadata",
                    "type": {
                        "option": {
                            "defined": {
                                "name": "depositMetadata"
                            }
                        }
                    }
                }
            ]
        },
        {
            "name": "userDepositSupportedToken",
            "discriminator": [
                139,
                84,
                137,
                218,
                229,
                151,
                183,
                154
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userFundAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "supportedTokenProgram"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount",
                        "userFundAccount",
                        "rewardAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "userReceiptTokenAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "supportedTokenMint"
                },
                {
                    "name": "fundSupportedTokenReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "userSupportedTokenAccount",
                    "writable": true
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "instructionsSysvar",
                    "address": "Sysvar1nstructions1111111111111111111111111"
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "amount",
                    "type": "u64"
                },
                {
                    "name": "metadata",
                    "type": {
                        "option": {
                            "defined": {
                                "name": "depositMetadata"
                            }
                        }
                    }
                }
            ]
        },
        {
            "name": "userInitializeFundAccount",
            "discriminator": [
                237,
                214,
                91,
                254,
                184,
                57,
                37,
                102
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint"
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "userReceiptTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "userInitializeRewardAccount",
            "discriminator": [
                35,
                77,
                53,
                232,
                36,
                237,
                146,
                246
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount"
                    ]
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "userReceiptTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "userRequestWithdrawal",
            "discriminator": [
                147,
                199,
                177,
                14,
                195,
                86,
                62,
                134
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userFundAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount",
                        "userFundAccount",
                        "rewardAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "receiptTokenLockAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "userReceiptTokenAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    114,
                                    101,
                                    115,
                                    101,
                                    114,
                                    118,
                                    101
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "instructionsSysvar",
                    "address": "Sysvar1nstructions1111111111111111111111111"
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "receiptTokenAmount",
                    "type": "u64"
                },
                {
                    "name": "supportedTokenMint",
                    "type": {
                        "option": "pubkey"
                    }
                }
            ]
        },
        {
            "name": "userUpdateFundAccountIfNeeded",
            "discriminator": [
                22,
                10,
                103,
                174,
                223,
                166,
                182,
                76
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userFundAccount"
                    ]
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "userFundAccount"
                    ]
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "userReceiptTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "userUpdateRewardAccountIfNeeded",
            "discriminator": [
                156,
                78,
                23,
                8,
                238,
                177,
                204,
                173
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount"
                    ]
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "userReceiptTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "desiredAccountSize",
                    "type": {
                        "option": "u32"
                    }
                }
            ]
        },
        {
            "name": "userUpdateRewardPools",
            "discriminator": [
                89,
                143,
                243,
                236,
                73,
                81,
                158,
                100
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "rewardAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": []
        },
        {
            "name": "userWithdrawSol",
            "discriminator": [
                214,
                13,
                137,
                164,
                194,
                105,
                183,
                252
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userFundAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "receiptTokenMint",
                    "relations": [
                        "fundAccount",
                        "fundWithdrawalBatchAccount",
                        "userFundAccount",
                        "rewardAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "userReceiptTokenAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    114,
                                    101,
                                    115,
                                    101,
                                    114,
                                    118,
                                    101
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundWithdrawalBatchAccount",
                    "docs": [
                        "Users can derive proper account address with target batch id for each withdrawal requests.",
                        "And the batch id can be read from a user fund account which the withdrawal requests belong to."
                    ],
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    119,
                                    105,
                                    116,
                                    104,
                                    100,
                                    114,
                                    97,
                                    119,
                                    97,
                                    108,
                                    95,
                                    98,
                                    97,
                                    116,
                                    99,
                                    104
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "const",
                                "value": [
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0
                                ]
                            },
                            {
                                "kind": "arg",
                                "path": "batchId"
                            }
                        ]
                    }
                },
                {
                    "name": "fundTreasuryAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    116,
                                    114,
                                    101,
                                    97,
                                    115,
                                    117,
                                    114,
                                    121
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "batchId",
                    "type": "u64"
                },
                {
                    "name": "requestId",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "userWithdrawSupportedToken",
            "discriminator": [
                95,
                90,
                176,
                21,
                252,
                231,
                133,
                99
            ],
            "accounts": [
                {
                    "name": "user",
                    "writable": true,
                    "signer": true,
                    "relations": [
                        "userFundAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "systemProgram",
                    "address": "11111111111111111111111111111111"
                },
                {
                    "name": "receiptTokenProgram",
                    "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                },
                {
                    "name": "supportedTokenProgram"
                },
                {
                    "name": "receiptTokenMint",
                    "writable": true,
                    "relations": [
                        "fundAccount",
                        "fundWithdrawalBatchAccount",
                        "userFundAccount",
                        "rewardAccount",
                        "userRewardAccount"
                    ]
                },
                {
                    "name": "userReceiptTokenAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "user"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "supportedTokenMint"
                },
                {
                    "name": "userSupportedTokenAccount",
                    "writable": true
                },
                {
                    "name": "fundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "fundWithdrawalBatchAccount",
                    "docs": [
                        "Users can derive proper account address with target batch id for each withdrawal requests.",
                        "And the batch id can be read from a user fund account which the withdrawal requests belong to."
                    ],
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    119,
                                    105,
                                    116,
                                    104,
                                    100,
                                    114,
                                    97,
                                    119,
                                    97,
                                    108,
                                    95,
                                    98,
                                    97,
                                    116,
                                    99,
                                    104
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenMint"
                            },
                            {
                                "kind": "arg",
                                "path": "batchId"
                            }
                        ]
                    }
                },
                {
                    "name": "fundSupportedTokenReserveAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "account",
                                "path": "fundAccount"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenProgram"
                            },
                            {
                                "kind": "account",
                                "path": "supportedTokenMint"
                            }
                        ],
                        "program": {
                            "kind": "const",
                            "value": [
                                140,
                                151,
                                37,
                                143,
                                78,
                                36,
                                137,
                                241,
                                187,
                                61,
                                16,
                                41,
                                20,
                                142,
                                13,
                                131,
                                11,
                                90,
                                19,
                                153,
                                218,
                                255,
                                16,
                                132,
                                4,
                                142,
                                123,
                                216,
                                219,
                                233,
                                248,
                                89
                            ]
                        }
                    }
                },
                {
                    "name": "fundTreasuryAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    102,
                                    117,
                                    110,
                                    100,
                                    95,
                                    116,
                                    114,
                                    101,
                                    97,
                                    115,
                                    117,
                                    114,
                                    121
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userFundAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    102,
                                    117,
                                    110,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "rewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            }
                        ]
                    }
                },
                {
                    "name": "userRewardAccount",
                    "writable": true,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    117,
                                    115,
                                    101,
                                    114,
                                    95,
                                    114,
                                    101,
                                    119,
                                    97,
                                    114,
                                    100
                                ]
                            },
                            {
                                "kind": "account",
                                "path": "receiptTokenMint"
                            },
                            {
                                "kind": "account",
                                "path": "user"
                            }
                        ]
                    }
                },
                {
                    "name": "instructionsSysvar",
                    "address": "Sysvar1nstructions1111111111111111111111111"
                },
                {
                    "name": "eventAuthority",
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "value": [
                                    95,
                                    95,
                                    101,
                                    118,
                                    101,
                                    110,
                                    116,
                                    95,
                                    97,
                                    117,
                                    116,
                                    104,
                                    111,
                                    114,
                                    105,
                                    116,
                                    121
                                ]
                            }
                        ]
                    }
                },
                {
                    "name": "program"
                }
            ],
            "args": [
                {
                    "name": "batchId",
                    "type": "u64"
                },
                {
                    "name": "requestId",
                    "type": "u64"
                }
            ]
        }
    ],
    "accounts": [
        {
            "name": "fundAccount",
            "discriminator": [
                49,
                104,
                168,
                214,
                134,
                180,
                173,
                154
            ]
        },
        {
            "name": "fundWithdrawalBatchAccount",
            "discriminator": [
                85,
                200,
                99,
                175,
                79,
                125,
                149,
                220
            ]
        },
        {
            "name": "normalizedTokenPoolAccount",
            "discriminator": [
                7,
                113,
                233,
                177,
                153,
                66,
                175,
                56
            ]
        },
        {
            "name": "normalizedTokenWithdrawalAccount",
            "discriminator": [
                93,
                156,
                243,
                244,
                209,
                190,
                231,
                249
            ]
        },
        {
            "name": "rewardAccount",
            "discriminator": [
                225,
                81,
                31,
                253,
                84,
                234,
                171,
                129
            ]
        },
        {
            "name": "userFundAccount",
            "discriminator": [
                208,
                166,
                47,
                241,
                179,
                76,
                157,
                212
            ]
        },
        {
            "name": "userRewardAccount",
            "discriminator": [
                55,
                245,
                122,
                238,
                147,
                89,
                164,
                198
            ]
        }
    ],
    "events": [
        {
            "name": "fundManagerUpdatedFund",
            "discriminator": [
                134,
                191,
                120,
                8,
                174,
                124,
                129,
                199
            ]
        },
        {
            "name": "fundManagerUpdatedRewardPool",
            "discriminator": [
                195,
                147,
                69,
                56,
                76,
                226,
                252,
                128
            ]
        },
        {
            "name": "operatorDonatedToFund",
            "discriminator": [
                87,
                48,
                245,
                185,
                4,
                76,
                165,
                242
            ]
        },
        {
            "name": "operatorRanFundCommand",
            "discriminator": [
                10,
                0,
                29,
                204,
                128,
                125,
                227,
                149
            ]
        },
        {
            "name": "operatorUpdatedFundPrices",
            "discriminator": [
                108,
                80,
                9,
                116,
                200,
                169,
                219,
                220
            ]
        },
        {
            "name": "operatorUpdatedNormalizedTokenPoolPrices",
            "discriminator": [
                45,
                104,
                4,
                51,
                239,
                13,
                241,
                0
            ]
        },
        {
            "name": "operatorUpdatedRewardPools",
            "discriminator": [
                105,
                173,
                28,
                190,
                209,
                115,
                63,
                91
            ]
        },
        {
            "name": "userCanceledWithdrawalRequestFromFund",
            "discriminator": [
                114,
                97,
                217,
                9,
                1,
                121,
                31,
                213
            ]
        },
        {
            "name": "userCreatedOrUpdatedFundAccount",
            "discriminator": [
                26,
                206,
                120,
                214,
                227,
                187,
                182,
                0
            ]
        },
        {
            "name": "userCreatedOrUpdatedRewardAccount",
            "discriminator": [
                41,
                212,
                58,
                138,
                122,
                212,
                165,
                155
            ]
        },
        {
            "name": "userDepositedToFund",
            "discriminator": [
                254,
                40,
                245,
                52,
                68,
                65,
                132,
                44
            ]
        },
        {
            "name": "userRequestedWithdrawalFromFund",
            "discriminator": [
                23,
                105,
                171,
                107,
                172,
                40,
                226,
                124
            ]
        },
        {
            "name": "userTransferredReceiptToken",
            "discriminator": [
                50,
                130,
                164,
                229,
                182,
                55,
                117,
                0
            ]
        },
        {
            "name": "userUpdatedRewardPool",
            "discriminator": [
                189,
                251,
                56,
                47,
                30,
                252,
                63,
                27
            ]
        },
        {
            "name": "userWithdrewFromFund",
            "discriminator": [
                158,
                87,
                58,
                31,
                154,
                207,
                166,
                164
            ]
        }
    ],
    "errors": [
        {
            "code": 6000,
            "name": "calculationArithmeticException",
            "msg": "calculation arithmetic exception"
        },
        {
            "code": 6001,
            "name": "indexOutOfBoundsException",
            "msg": "index out of bounds exception"
        },
        {
            "code": 6002,
            "name": "utf8DecodingException",
            "msg": "utf-8 decoding exception"
        },
        {
            "code": 6003,
            "name": "invalidSignatureError",
            "msg": "signature verification failed"
        },
        {
            "code": 6004,
            "name": "invalidAccountDataVersionError",
            "msg": "invalid account data version"
        },
        {
            "code": 6005,
            "name": "tokenNotTransferableError",
            "msg": "token is not transferable currently"
        },
        {
            "code": 6006,
            "name": "tokenNotTransferringException",
            "msg": "token is not transferring currently"
        },
        {
            "code": 6007,
            "name": "rewardInvalidTransferArgsException",
            "msg": "reward: invalid token transfer args"
        },
        {
            "code": 6008,
            "name": "rewardInvalidMetadataNameLengthError",
            "msg": "reward: invalid metadata name length"
        },
        {
            "code": 6009,
            "name": "rewardInvalidMetadataDescriptionLengthError",
            "msg": "reward: invalid metadata description length"
        },
        {
            "code": 6010,
            "name": "rewardInvalidRewardTypeError",
            "msg": "reward: invalid reward type"
        },
        {
            "code": 6011,
            "name": "rewardAlreadyExistingHolderError",
            "msg": "reward: already existing holder"
        },
        {
            "code": 6012,
            "name": "rewardAlreadyExistingRewardError",
            "msg": "reward: already existing reward"
        },
        {
            "code": 6013,
            "name": "rewardAlreadyExistingPoolError",
            "msg": "reward: already existing pool"
        },
        {
            "code": 6014,
            "name": "rewardHolderNotFoundError",
            "msg": "reward: holder not found"
        },
        {
            "code": 6015,
            "name": "rewardNotFoundError",
            "msg": "reward: reward not found"
        },
        {
            "code": 6016,
            "name": "rewardPoolNotFoundError",
            "msg": "reward: pool not found"
        },
        {
            "code": 6017,
            "name": "rewardUserPoolNotFoundError",
            "msg": "reward: user pool not found"
        },
        {
            "code": 6018,
            "name": "rewardPoolClosedError",
            "msg": "reward: pool is closed"
        },
        {
            "code": 6019,
            "name": "rewardInvalidPoolConfigurationException",
            "msg": "reward: invalid pool configuration"
        },
        {
            "code": 6020,
            "name": "rewardInvalidPoolAccessException",
            "msg": "reward: invalid reward pool access"
        },
        {
            "code": 6021,
            "name": "rewardInvalidAccountingException",
            "msg": "reward: incorrect accounting exception"
        },
        {
            "code": 6022,
            "name": "rewardInvalidAllocatedAmountDeltaException",
            "msg": "reward: invalid amount or contribution accrual rate"
        },
        {
            "code": 6023,
            "name": "rewardExceededMaxHoldersError",
            "msg": "reward: exceeded max holders"
        },
        {
            "code": 6024,
            "name": "rewardExceededMaxRewardsError",
            "msg": "reward: exceeded max rewards"
        },
        {
            "code": 6025,
            "name": "rewardExceededMaxRewardPoolsError",
            "msg": "reward: exceeded max reward pools"
        },
        {
            "code": 6026,
            "name": "rewardExceededMaxUserRewardPoolsError",
            "msg": "reward: exceeded max user reward pools"
        },
        {
            "code": 6027,
            "name": "rewardExceededMaxHolderPubkeysError",
            "msg": "reward: exceeded max pubkeys per holder"
        },
        {
            "code": 6028,
            "name": "rewardExceededMaxTokenAllocatedAmountRecordException",
            "msg": "reward: exceeded max token allocated amount record"
        },
        {
            "code": 6029,
            "name": "rewardExceededMaxRewardSettlementError",
            "msg": "reward: exceeded max reward settlements per pool"
        },
        {
            "code": 6030,
            "name": "rewardStaleSettlementBlockNotExistError",
            "msg": "reward: stale settlement block not exist"
        },
        {
            "code": 6031,
            "name": "rewardInvalidSettlementBlockHeightException",
            "msg": "reward: invalid settlement block height"
        },
        {
            "code": 6032,
            "name": "rewardInvalidSettlementBlockContributionException",
            "msg": "reward: invalid settlement block contribution"
        },
        {
            "code": 6033,
            "name": "rewardInvalidTotalUserSettledAmountException",
            "msg": "reward: sum of user settled amount cannot exceed total amount"
        },
        {
            "code": 6034,
            "name": "rewardInvalidTotalUserSettledContributionException",
            "msg": "reward: sum of user settled contribution cannot exceed total contribution"
        },
        {
            "code": 6035,
            "name": "rewardPoolCloseConditionError",
            "msg": "reward: cannot close the reward pool"
        },
        {
            "code": 6036,
            "name": "tokenPricingSourceAccountNotFoundError",
            "msg": "pricing: token pricing source is not found"
        },
        {
            "code": 6037,
            "name": "fundInvalidConfigurationUpdateError",
            "msg": "fund: cannot apply invalid configuration update"
        },
        {
            "code": 6038,
            "name": "fundAlreadySupportedTokenError",
            "msg": "fund: already supported token"
        },
        {
            "code": 6039,
            "name": "fundNotSupportedTokenError",
            "msg": "fund: not supported token"
        },
        {
            "code": 6040,
            "name": "fundDepositDisabledError",
            "msg": "fund: deposit is currently disabled"
        },
        {
            "code": 6041,
            "name": "fundExceededDepositCapacityAmountError",
            "msg": "fund: exceeded deposit capacity amount"
        },
        {
            "code": 6042,
            "name": "fundDepositNotSupportedAsset",
            "msg": "fund: deposit is not supported for the given asset"
        },
        {
            "code": 6043,
            "name": "fundExceededMaxWithdrawalRequestError",
            "msg": "fund: exceeded max withdrawal request per user"
        },
        {
            "code": 6044,
            "name": "fundWithdrawalRequestNotFoundError",
            "msg": "fund: withdrawal request not found"
        },
        {
            "code": 6045,
            "name": "fundWithdrawalRequestIncorrectBatchError",
            "msg": "fund: withdrawal request not belongs to the given batch"
        },
        {
            "code": 6046,
            "name": "fundWithdrawalDisabledError",
            "msg": "fund: withdrawal is currently disabled"
        },
        {
            "code": 6047,
            "name": "fundWithdrawalNotSupportedAsset",
            "msg": "fund: withdrawal is not supported for the given asset"
        },
        {
            "code": 6048,
            "name": "fundWithdrawalReserveExhaustedSupportedAsset",
            "msg": "fund: withdrawal reserve is exhausted for the given asset"
        },
        {
            "code": 6049,
            "name": "fundWithdrawalRequestAlreadyQueuedError",
            "msg": "fund: withdrawal request is already in progress"
        },
        {
            "code": 6050,
            "name": "fundDepositMetadataSignatureExpiredError",
            "msg": "fund: deposit metadata signature has expired"
        },
        {
            "code": 6051,
            "name": "fundExceededMaxSupportedTokensError",
            "msg": "fund: exceeded max supported tokens"
        },
        {
            "code": 6052,
            "name": "fundInvalidWithdrawalFeeRateError",
            "msg": "fund: invalid withdrawal fee rate"
        },
        {
            "code": 6053,
            "name": "fundNormalizedTokenAlreadySetError",
            "msg": "fund: normalized token already set"
        },
        {
            "code": 6054,
            "name": "fundNormalizedTokenNotSetError",
            "msg": "fund: normalized token is not set"
        },
        {
            "code": 6055,
            "name": "fundRestakingVaultAlreadyRegisteredError",
            "msg": "fund: restaking vault already registered"
        },
        {
            "code": 6056,
            "name": "fundExceededMaxRestakingVaultsError",
            "msg": "reward: exceeded max restaking vaults"
        },
        {
            "code": 6057,
            "name": "fundRestakingNotSupportedVaultError",
            "msg": "fund: not supported restaking vault"
        },
        {
            "code": 6058,
            "name": "fundRestakingVaultNotFoundError",
            "msg": "fund: restaking vault not found"
        },
        {
            "code": 6059,
            "name": "fundRestakingVaultOperatorNotFoundError",
            "msg": "fund: restaking vault operator not found"
        },
        {
            "code": 6060,
            "name": "fundRestakingVaultOperatorAlreadyRegisteredError",
            "msg": "fund: restaking vault operator already registered"
        },
        {
            "code": 6061,
            "name": "fundExceededMaxRestakingVaultDelegationsError",
            "msg": "fund: exceeded max restaking vault delegations"
        },
        {
            "code": 6062,
            "name": "fundOperationUnauthorizedCommandError",
            "msg": "fund: unauhorized operation command"
        },
        {
            "code": 6063,
            "name": "fundOperationCommandAccountComputationException",
            "msg": "fund: failed to compute required accounts for the operation command"
        },
        {
            "code": 6064,
            "name": "fundOperationCommandExecutionFailedException",
            "msg": "fund: failed to execute the operation command"
        },
        {
            "code": 6065,
            "name": "normalizedTokenPoolNotSupportedTokenError",
            "msg": "normalization: not supported token"
        },
        {
            "code": 6066,
            "name": "normalizedTokenPoolAlreadySupportedTokenError",
            "msg": "normalization: already supported token"
        },
        {
            "code": 6067,
            "name": "normalizedTokenPoolExceededMaxSupportedTokensError",
            "msg": "normalization: exceeded max supported tokens"
        },
        {
            "code": 6068,
            "name": "normalizedTokenPoolNotEnoughSupportedTokenException",
            "msg": "normalization: not enough supported token in the pool"
        },
        {
            "code": 6069,
            "name": "normalizedTokenPoolAlreadySettledWithdrawalAccountError",
            "msg": "normalization: already settled withdrawal account"
        },
        {
            "code": 6070,
            "name": "normalizedTokenPoolNonClaimableTokenError",
            "msg": "normalization: the token is non-claimable for the given withdrawal account"
        },
        {
            "code": 6071,
            "name": "stakingUninitializedWithdrawTicketNotFoundException",
            "msg": "staking: failed to find uninitialized withdraw ticket"
        },
        {
            "code": 6072,
            "name": "stakingAccountNotMatchedException",
            "msg": "staking: account not matched"
        },
        {
            "code": 6073,
            "name": "stakingSplActiveStakeNotAvailableException",
            "msg": "staking: spl stake pool's active stake not available"
        },
        {
            "code": 6074,
            "name": "restakingVaultWithdrawalTicketsExhaustedError",
            "msg": "restaking: all withdrawal tickets are already in use"
        },
        {
            "code": 6075,
            "name": "restakingVaultWithdrawalTicketNotWithdrawableError",
            "msg": "restaking: withdrawal ticket is not withdrawable"
        },
        {
            "code": 6076,
            "name": "restakingVaultWithdrawalTicketAlreadyInitializedError",
            "msg": "restaking: withdrawal ticket is already initialized"
        }
    ],
    "types": [
        {
            "name": "asset",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "sol",
                        "fields": [
                            "u64"
                        ]
                    },
                    {
                        "name": "token",
                        "fields": [
                            "pubkey",
                            {
                                "option": {
                                    "defined": {
                                        "name": "tokenPricingSource"
                                    }
                                }
                            },
                            "u64"
                        ]
                    }
                ]
            }
        },
        {
            "name": "assetPod",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "discriminant",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    },
                    {
                        "name": "solAmount",
                        "type": "u64"
                    },
                    {
                        "name": "tokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "tokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "tokenPricingSource",
                        "type": {
                            "defined": {
                                "name": "tokenPricingSourcePod"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "assetState",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "tokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "tokenProgram",
                        "type": "pubkey"
                    },
                    {
                        "name": "accumulatedDepositCapacityAmount",
                        "type": "u64"
                    },
                    {
                        "name": "accumulatedDepositAmount",
                        "type": "u64"
                    },
                    {
                        "name": "depositable",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "withdrawable",
                        "type": "u8"
                    },
                    {
                        "name": "normalReserveRateBps",
                        "type": "u16"
                    },
                    {
                        "name": "normalReserveMaxAmount",
                        "type": "u64"
                    },
                    {
                        "name": "withdrawalLastCreatedRequestId",
                        "type": "u64"
                    },
                    {
                        "name": "withdrawalLastProcessedBatchId",
                        "type": "u64"
                    },
                    {
                        "name": "withdrawalLastBatchEnqueuedAt",
                        "type": "i64"
                    },
                    {
                        "name": "withdrawalLastBatchProcessedAt",
                        "type": "i64"
                    },
                    {
                        "name": "withdrawalPendingBatch",
                        "type": {
                            "defined": {
                                "name": "withdrawalBatch"
                            }
                        }
                    },
                    {
                        "name": "padding2",
                        "type": {
                            "array": [
                                "u8",
                                15
                            ]
                        }
                    },
                    {
                        "name": "withdrawalNumQueuedBatches",
                        "type": "u8"
                    },
                    {
                        "name": "withdrawalQueuedBatches",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "withdrawalBatch"
                                    }
                                },
                                10
                            ]
                        }
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                56
                            ]
                        }
                    },
                    {
                        "name": "withdrawableValueAsReceiptTokenAmount",
                        "docs": [
                            "receipt token amount that users can request to withdraw with the given asset from the fund.",
                            "it can be conditionally inaccurate on price changes among multiple assets, so make sure to update this properly before any use of it.",
                            "do not make any hard limit constraints with this value from off-chain. a requested withdrawal amount will be adjusted on-chain based on the status."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "withdrawalUserReservedAmount",
                        "docs": [
                            "informative: reserved amount that users can claim for processed withdrawal requests, which is not accounted for as an asset of the fund."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "operationReceivableAmount",
                        "docs": [
                            "asset: receivable amount that the fund may charge the users requesting withdrawals.",
                            "It is accrued during either the preparation of the withdrawal obligation or rebalancing of LST like fees from (un)staking or (un)restaking.",
                            "And it shall be settled by the withdrawal fee normally. And it also can be written off by a donation operation.",
                            "Then it costs the rebalancing expense to the capital of the fund itself as an operation cost instead of charging the users requesting withdrawals."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "operationReservedAmount",
                        "docs": [
                            "asset: remaining asset for cash-in/out"
                        ],
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "claimUnrestakedVstCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "items",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "claimUnrestakedVstCommandItem"
                                }
                            }
                        }
                    },
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "claimUnrestakedVstCommandState"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "claimUnrestakedVstCommandItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "vaultAddress",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "claimUnrestakedVstCommandResult",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "claimUnrestakedVstCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "init"
                    },
                    {
                        "name": "init2"
                    },
                    {
                        "name": "readVaultState"
                    },
                    {
                        "name": "claim",
                        "fields": [
                            {
                                "defined": {
                                    "name": "claimableUnrestakeWithdrawalStatus"
                                }
                            }
                        ]
                    },
                    {
                        "name": "setupDenormalize",
                        "fields": [
                            "u64"
                        ]
                    },
                    {
                        "name": "denormalize",
                        "fields": [
                            {
                                "vec": {
                                    "defined": {
                                        "name": "denormalizeSupportedTokenAsset"
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "claimUnstakedSolCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "items",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "claimUnstakedSolCommandItem"
                                }
                            }
                        }
                    },
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "claimUnstakedSolCommandState"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "claimUnstakedSolCommandItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "mint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundStakeAccounts",
                        "type": {
                            "vec": "pubkey"
                        }
                    }
                ]
            }
        },
        {
            "name": "claimUnstakedSolCommandResult",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "claimUnstakedSolCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "init"
                    },
                    {
                        "name": "readPoolState"
                    },
                    {
                        "name": "claim"
                    }
                ]
            }
        },
        {
            "name": "claimableUnrestakeWithdrawalStatus",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "withdrawalTickets",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "claimableUnrestakeWithdrawalTicket"
                                }
                            }
                        }
                    },
                    {
                        "name": "expectedNcnEpoch",
                        "type": "u64"
                    },
                    {
                        "name": "delayedNcnEpoch",
                        "type": "u64"
                    },
                    {
                        "name": "unrestakedVstAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "claimableUnrestakeWithdrawalTicket",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "withdrawalTicketAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "withdrawalTicketTokenAccount",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "delegateVstCommand",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "delegateVstCommandResult",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "denormalizeNtCommand",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "denormalizeNtCommandResult",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "denormalizeSupportedTokenAsset",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "operationReservedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "tokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "tokenProgram",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "depositMetadata",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "user",
                        "type": "pubkey"
                    },
                    {
                        "name": "walletProvider",
                        "type": "string"
                    },
                    {
                        "name": "contributionAccrualRate",
                        "type": "u8"
                    },
                    {
                        "name": "expiredAt",
                        "type": "i64"
                    }
                ]
            }
        },
        {
            "name": "enqueueWithdrawalBatchCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "forced",
                        "type": "bool"
                    }
                ]
            }
        },
        {
            "name": "enqueueWithdrawalBatchCommandResult",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "enqueuedReceiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "totalQueuedReceiptTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "fundAccount",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "dataVersion",
                        "type": "u16"
                    },
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "reserveAccountBump",
                        "type": "u8"
                    },
                    {
                        "name": "treasuryAccountBump",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                10
                            ]
                        }
                    },
                    {
                        "name": "transferEnabled",
                        "type": "u8"
                    },
                    {
                        "name": "receiptTokenMint",
                        "docs": [
                            "receipt token information"
                        ],
                        "type": "pubkey"
                    },
                    {
                        "name": "receiptTokenProgram",
                        "type": "pubkey"
                    },
                    {
                        "name": "receiptTokenDecimals",
                        "type": "u8"
                    },
                    {
                        "name": "padding2",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    },
                    {
                        "name": "receiptTokenSupplyAmount",
                        "type": "u64"
                    },
                    {
                        "name": "oneReceiptTokenAsSol",
                        "type": "u64"
                    },
                    {
                        "name": "receiptTokenValueUpdatedSlot",
                        "type": "u64"
                    },
                    {
                        "name": "receiptTokenValue",
                        "type": {
                            "defined": {
                                "name": "tokenValuePod"
                            }
                        }
                    },
                    {
                        "name": "withdrawalBatchThresholdIntervalSeconds",
                        "docs": [
                            "global withdrawal configurations"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "withdrawalFeeRateBps",
                        "type": "u16"
                    },
                    {
                        "name": "withdrawalEnabled",
                        "type": "u8"
                    },
                    {
                        "name": "depositEnabled",
                        "type": "u8"
                    },
                    {
                        "name": "padding4",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "sol",
                        "docs": [
                            "SOL deposit & withdrawal"
                        ],
                        "type": {
                            "defined": {
                                "name": "assetState"
                            }
                        }
                    },
                    {
                        "name": "padding6",
                        "docs": [
                            "underlying assets"
                        ],
                        "type": {
                            "array": [
                                "u8",
                                15
                            ]
                        }
                    },
                    {
                        "name": "numSupportedTokens",
                        "type": "u8"
                    },
                    {
                        "name": "supportedTokens",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "supportedToken"
                                    }
                                },
                                30
                            ]
                        }
                    },
                    {
                        "name": "normalizedToken",
                        "docs": [
                            "optional basket of underlying assets"
                        ],
                        "type": {
                            "defined": {
                                "name": "normalizedToken"
                            }
                        }
                    },
                    {
                        "name": "padding7",
                        "docs": [
                            "investments"
                        ],
                        "type": {
                            "array": [
                                "u8",
                                15
                            ]
                        }
                    },
                    {
                        "name": "numRestakingVaults",
                        "type": "u8"
                    },
                    {
                        "name": "restakingVaults",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "restakingVault"
                                    }
                                },
                                30
                            ]
                        }
                    },
                    {
                        "name": "operation",
                        "docs": [
                            "fund operation state"
                        ],
                        "type": {
                            "defined": {
                                "name": "operationState"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "fundManagerUpdatedFund",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "fundManagerUpdatedRewardPool",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "rewardAccount",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "fundWithdrawalBatchAccount",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "dataVersion",
                        "type": "u16"
                    },
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "supportedTokenProgram",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "batchId",
                        "type": "u64"
                    },
                    {
                        "name": "numRequests",
                        "type": "u64"
                    },
                    {
                        "name": "numClaimedRequests",
                        "type": "u64"
                    },
                    {
                        "name": "receiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "claimedReceiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "assetUserAmount",
                        "docs": [
                            "asset to be withdrawn"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "claimedAssetUserAmount",
                        "type": "u64"
                    },
                    {
                        "name": "assetFeeAmount",
                        "docs": [
                            "informative: withdrawal fee is already paid to the treasury account, just informative."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "processedAt",
                        "type": "i64"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                32
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "harvestRewardCommand",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "harvestRewardCommandResult",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "initializeCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "initializeCommandState"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "initializeCommandRestakingVaultUpdateItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "vault",
                        "type": "pubkey"
                    },
                    {
                        "name": "delegationsUpdatedBitmap",
                        "type": {
                            "vec": "bool"
                        }
                    }
                ]
            }
        },
        {
            "name": "initializeCommandResult",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "restakingVaultUpdated",
                        "type": {
                            "option": {
                                "defined": {
                                    "name": "initializeCommandResultRestakingVaultUpdated"
                                }
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "initializeCommandResultRestakingVaultDelegationUpdate",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "operator",
                        "type": "pubkey"
                    },
                    {
                        "name": "delegatedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "undelegatingAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "initializeCommandResultRestakingVaultUpdated",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "vault",
                        "type": "pubkey"
                    },
                    {
                        "name": "epoch",
                        "type": "u64"
                    },
                    {
                        "name": "finalized",
                        "type": "bool"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "delegations",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "initializeCommandResultRestakingVaultDelegationUpdate"
                                }
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "initializeCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "new"
                    },
                    {
                        "name": "prepareSingleRestakingVaultUpdate",
                        "fields": [
                            {
                                "name": "vault",
                                "type": "pubkey"
                            },
                            {
                                "name": "operator",
                                "type": "pubkey"
                            }
                        ]
                    },
                    {
                        "name": "prepareRestakingVaultUpdate",
                        "fields": [
                            {
                                "name": "items",
                                "type": {
                                    "vec": {
                                        "defined": {
                                            "name": "initializeCommandRestakingVaultUpdateItem"
                                        }
                                    }
                                }
                            }
                        ]
                    },
                    {
                        "name": "executeRestakingVaultUpdate",
                        "fields": [
                            {
                                "name": "items",
                                "type": {
                                    "vec": {
                                        "defined": {
                                            "name": "initializeCommandRestakingVaultUpdateItem"
                                        }
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "normalizeStCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "normalizeStCommandState"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "normalizeStCommandItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "supportedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "allocatedTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "normalizeStCommandResult",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "supportedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "normalizedSupportedTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "mintedTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "operationReservedTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "normalizeStCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "new"
                    },
                    {
                        "name": "prepare",
                        "fields": [
                            {
                                "name": "items",
                                "type": {
                                    "vec": {
                                        "defined": {
                                            "name": "normalizeStCommandItem"
                                        }
                                    }
                                }
                            }
                        ]
                    },
                    {
                        "name": "execute",
                        "fields": [
                            {
                                "name": "items",
                                "type": {
                                    "vec": {
                                        "defined": {
                                            "name": "normalizeStCommandItem"
                                        }
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "normalizedClaimableToken",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "mint",
                        "type": "pubkey"
                    },
                    {
                        "name": "program",
                        "type": "pubkey"
                    },
                    {
                        "name": "claimableAmount",
                        "type": "u64"
                    },
                    {
                        "name": "claimed",
                        "type": "bool"
                    }
                ]
            }
        },
        {
            "name": "normalizedSupportedToken",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "mint",
                        "type": "pubkey"
                    },
                    {
                        "name": "program",
                        "type": "pubkey"
                    },
                    {
                        "name": "reserveAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "lockedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "decimals",
                        "type": "u8"
                    },
                    {
                        "name": "withdrawalReservedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "oneTokenAsSol",
                        "type": "u64"
                    },
                    {
                        "name": "pricingSource",
                        "type": {
                            "defined": {
                                "name": "tokenPricingSource"
                            }
                        }
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                14
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "normalizedToken",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "mint",
                        "type": "pubkey"
                    },
                    {
                        "name": "program",
                        "type": "pubkey"
                    },
                    {
                        "name": "decimals",
                        "type": "u8"
                    },
                    {
                        "name": "enabled",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                6
                            ]
                        }
                    },
                    {
                        "name": "pricingSource",
                        "type": {
                            "defined": {
                                "name": "tokenPricingSourcePod"
                            }
                        }
                    },
                    {
                        "name": "oneTokenAsSol",
                        "type": "u64"
                    },
                    {
                        "name": "operationReservedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                64
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "normalizedTokenPoolAccount",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "dataVersion",
                        "type": "u16"
                    },
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "normalizedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "normalizedTokenProgram",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokens",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "normalizedSupportedToken"
                                }
                            }
                        }
                    },
                    {
                        "name": "normalizedTokenDecimals",
                        "type": "u8"
                    },
                    {
                        "name": "normalizedTokenSupplyAmount",
                        "type": "u64"
                    },
                    {
                        "name": "normalizedTokenValue",
                        "type": {
                            "defined": {
                                "name": "tokenValue"
                            }
                        }
                    },
                    {
                        "name": "normalizedTokenValueUpdatedSlot",
                        "type": "u64"
                    },
                    {
                        "name": "oneNormalizedTokenAsSol",
                        "type": "u64"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                128
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "normalizedTokenWithdrawalAccount",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "dataVersion",
                        "type": "u16"
                    },
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "withdrawalAuthority",
                        "type": "pubkey"
                    },
                    {
                        "name": "normalizedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "normalizedTokenPool",
                        "type": "pubkey"
                    },
                    {
                        "name": "normalizedTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "claimableTokens",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "normalizedClaimableToken"
                                }
                            }
                        }
                    },
                    {
                        "name": "createdAt",
                        "type": "i64"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                32
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "operationCommand",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "initialize",
                        "fields": [
                            {
                                "defined": {
                                    "name": "initializeCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "enqueueWithdrawalBatch",
                        "fields": [
                            {
                                "defined": {
                                    "name": "enqueueWithdrawalBatchCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "claimUnrestakedVst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "claimUnrestakedVstCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "denormalizeNt",
                        "fields": [
                            {
                                "defined": {
                                    "name": "denormalizeNtCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "undelegateVst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "undelegateVstCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "unrestakeVrt",
                        "fields": [
                            {
                                "defined": {
                                    "name": "unrestakeVrtCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "claimUnstakedSol",
                        "fields": [
                            {
                                "defined": {
                                    "name": "claimUnstakedSolCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "unstakeLst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "unstakeLstCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "processWithdrawalBatch",
                        "fields": [
                            {
                                "defined": {
                                    "name": "processWithdrawalBatchCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "stakeSol",
                        "fields": [
                            {
                                "defined": {
                                    "name": "stakeSolCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "normalizeSt",
                        "fields": [
                            {
                                "defined": {
                                    "name": "normalizeStCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "restakeVst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "restakeVstCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "delegateVst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "delegateVstCommand"
                                }
                            }
                        ]
                    },
                    {
                        "name": "harvestReward",
                        "fields": [
                            {
                                "defined": {
                                    "name": "harvestRewardCommand"
                                }
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "operationCommandAccountMeta",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "pubkey",
                        "type": "pubkey"
                    },
                    {
                        "name": "isWritable",
                        "type": "bool"
                    }
                ]
            }
        },
        {
            "name": "operationCommandAccountMetaPod",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "pubkey",
                        "type": "pubkey"
                    },
                    {
                        "name": "isWritable",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "operationCommandEntry",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "command",
                        "type": {
                            "defined": {
                                "name": "operationCommand"
                            }
                        }
                    },
                    {
                        "name": "requiredAccounts",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "operationCommandAccountMeta"
                                }
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "operationCommandEntryPod",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "numRequiredAccounts",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    },
                    {
                        "name": "requiredAccounts",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "operationCommandAccountMetaPod"
                                    }
                                },
                                32
                            ]
                        }
                    },
                    {
                        "name": "command",
                        "type": {
                            "defined": {
                                "name": "operationCommandPod"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "operationCommandPod",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "discriminant",
                        "type": "u8"
                    },
                    {
                        "name": "buffer",
                        "type": {
                            "array": [
                                "u8",
                                2535
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "operationCommandResult",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "initialize",
                        "fields": [
                            {
                                "defined": {
                                    "name": "initializeCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "enqueueWithdrawalBatch",
                        "fields": [
                            {
                                "defined": {
                                    "name": "enqueueWithdrawalBatchCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "claimUnrestakedVst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "claimUnrestakedVstCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "denormalizeNt",
                        "fields": [
                            {
                                "defined": {
                                    "name": "denormalizeNtCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "undelegateVst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "undelegateVstCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "unrestakeVrt",
                        "fields": [
                            {
                                "defined": {
                                    "name": "unrestakeVrtCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "claimUnstakedSol",
                        "fields": [
                            {
                                "defined": {
                                    "name": "claimUnstakedSolCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "unstakeLst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "unstakeLstCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "processWithdrawalBatch",
                        "fields": [
                            {
                                "defined": {
                                    "name": "processWithdrawalBatchCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "stakeSol",
                        "fields": [
                            {
                                "defined": {
                                    "name": "stakeSolCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "normalizeSt",
                        "fields": [
                            {
                                "defined": {
                                    "name": "normalizeStCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "restakeVst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "restakeVstCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "delegateVst",
                        "fields": [
                            {
                                "defined": {
                                    "name": "delegateVstCommandResult"
                                }
                            }
                        ]
                    },
                    {
                        "name": "harvestReward",
                        "fields": [
                            {
                                "defined": {
                                    "name": "harvestRewardCommandResult"
                                }
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "operationState",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "updatedSlot",
                        "type": "u64"
                    },
                    {
                        "name": "updatedAt",
                        "type": "i64"
                    },
                    {
                        "name": "expiredAt",
                        "type": "i64"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                5
                            ]
                        }
                    },
                    {
                        "name": "noTransition",
                        "docs": [
                            "when the no_transition flag turned on, current command should not be transitioned to other command.",
                            "the purpose of this flag is for internal testing by set boundary of the reset command operation."
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "nextSequence",
                        "type": "u16"
                    },
                    {
                        "name": "numOperated",
                        "type": "u64"
                    },
                    {
                        "name": "nextCommand",
                        "type": {
                            "defined": {
                                "name": "operationCommandEntryPod"
                            }
                        }
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                128
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "operatorDonatedToFund",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "donatedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "depositedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "offsettedReceivableAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "operatorRanFundCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "nextSequence",
                        "type": "u16"
                    },
                    {
                        "name": "numOperated",
                        "type": "u64"
                    },
                    {
                        "name": "command",
                        "type": {
                            "defined": {
                                "name": "operationCommand"
                            }
                        }
                    },
                    {
                        "name": "result",
                        "type": {
                            "option": {
                                "defined": {
                                    "name": "operationCommandResult"
                                }
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "operatorUpdatedFundPrices",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "operatorUpdatedNormalizedTokenPoolPrices",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "normalizedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "normalizedTokenPoolAccount",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "operatorUpdatedRewardPools",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "rewardAccount",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "processWithdrawalBatchCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "processWithdrawalBatchCommandState"
                            }
                        }
                    },
                    {
                        "name": "forced",
                        "type": "bool"
                    }
                ]
            }
        },
        {
            "name": "processWithdrawalBatchCommandResult",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "requestedReceiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "processedReceiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "assetTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "reservedAssetUserAmount",
                        "type": "u64"
                    },
                    {
                        "name": "deductedAssetFeeAmount",
                        "type": "u64"
                    },
                    {
                        "name": "offsettedAssetReceivables",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "processWithdrawalBatchCommandResultAssetReceivable"
                                }
                            }
                        }
                    },
                    {
                        "name": "transferredAssetRevenueAmount",
                        "type": "u64"
                    },
                    {
                        "name": "withdrawalFeeRateBps",
                        "type": "u16"
                    }
                ]
            }
        },
        {
            "name": "processWithdrawalBatchCommandResultAssetReceivable",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "assetTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "assetAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "processWithdrawalBatchCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "new"
                    },
                    {
                        "name": "prepare",
                        "fields": [
                            {
                                "name": "assetTokenMint",
                                "type": {
                                    "option": "pubkey"
                                }
                            }
                        ]
                    },
                    {
                        "name": "execute",
                        "fields": [
                            {
                                "name": "assetTokenMint",
                                "type": {
                                    "option": "pubkey"
                                }
                            },
                            {
                                "name": "numProcessingBatches",
                                "type": "u8"
                            },
                            {
                                "name": "receiptTokenAmount",
                                "type": "u64"
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "restakeVstCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "restakeVstCommandState"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "restakeVstCommandItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "vault",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "allocatedTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "restakeVstCommandResult",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "supportedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "depositedSupportedTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "deductedSupportedTokenFeeAmount",
                        "type": "u64"
                    },
                    {
                        "name": "mintedTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "operationReservedTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "restakeVstCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "new"
                    },
                    {
                        "name": "prepare",
                        "fields": [
                            {
                                "name": "items",
                                "type": {
                                    "vec": {
                                        "defined": {
                                            "name": "restakeVstCommandItem"
                                        }
                                    }
                                }
                            }
                        ]
                    },
                    {
                        "name": "execute",
                        "fields": [
                            {
                                "name": "items",
                                "type": {
                                    "vec": {
                                        "defined": {
                                            "name": "restakeVstCommandItem"
                                        }
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "restakingVault",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "vault",
                        "type": "pubkey"
                    },
                    {
                        "name": "program",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "receiptTokenProgram",
                        "type": "pubkey"
                    },
                    {
                        "name": "receiptTokenDecimals",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    },
                    {
                        "name": "oneReceiptTokenAsSol",
                        "docs": [
                            "transient price"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "receiptTokenPricingSource",
                        "type": {
                            "defined": {
                                "name": "tokenPricingSourcePod"
                            }
                        }
                    },
                    {
                        "name": "receiptTokenOperationReservedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "receiptTokenOperationReceivableAmount",
                        "docs": [
                            "the amount of vrt being unrestaked"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "solAllocationWeight",
                        "docs": [
                            "configuration: used for restaking allocation strategy."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "solAllocationCapacityAmount",
                        "type": "u64"
                    },
                    {
                        "name": "padding2",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    },
                    {
                        "name": "numDelegations",
                        "type": "u8"
                    },
                    {
                        "name": "delegations",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "restakingVaultDelegation"
                                    }
                                },
                                30
                            ]
                        }
                    },
                    {
                        "name": "compoundingRewardTokenMints",
                        "docs": [
                            "auto-compounding"
                        ],
                        "type": {
                            "array": [
                                "pubkey",
                                10
                            ]
                        }
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                128
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "restakingVaultDelegation",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "operator",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenAllocationWeight",
                        "docs": [
                            "configuration: used for delegation strategy."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "supportedTokenAllocationCapacityAmount",
                        "type": "u64"
                    },
                    {
                        "name": "supportedTokenDelegatedAmount",
                        "docs": [
                            "informative field; these values shall be synced from remote state periodically."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "supportedTokenUndelegatingAmount",
                        "type": "u64"
                    },
                    {
                        "name": "supportedTokenRedelegatingAmount",
                        "docs": [
                            "configuration: the amount requested to be undelegated as soon as possible regardless of current state, this value should be decreased by each undelegation requested amount."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                24
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "reward",
            "docs": [
                "Reward type."
            ],
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "id",
                        "docs": [
                            "ID is determined by reward account."
                        ],
                        "type": "u16"
                    },
                    {
                        "name": "name",
                        "type": {
                            "array": [
                                "u8",
                                14
                            ]
                        }
                    },
                    {
                        "name": "description",
                        "type": {
                            "array": [
                                "u8",
                                128
                            ]
                        }
                    },
                    {
                        "name": "rewardTypeDiscriminant",
                        "type": "u8"
                    },
                    {
                        "name": "tokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "tokenProgram",
                        "type": "pubkey"
                    },
                    {
                        "name": "decimals",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                14
                            ]
                        }
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u64",
                                16
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "rewardAccount",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "dataVersion",
                        "type": "u16"
                    },
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "maxHolders",
                        "type": "u8"
                    },
                    {
                        "name": "maxRewards",
                        "type": "u16"
                    },
                    {
                        "name": "maxRewardPools",
                        "type": "u8"
                    },
                    {
                        "name": "numHolders",
                        "type": "u8"
                    },
                    {
                        "name": "numRewards",
                        "type": "u16"
                    },
                    {
                        "name": "numRewardPools",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                5
                            ]
                        }
                    },
                    {
                        "name": "holders1",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "rewardPoolHolder"
                                    }
                                },
                                4
                            ]
                        }
                    },
                    {
                        "name": "rewards1",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "reward"
                                    }
                                },
                                16
                            ]
                        }
                    },
                    {
                        "name": "rewardPools1",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "rewardPool"
                                    }
                                },
                                4
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "rewardPool",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "id",
                        "docs": [
                            "ID is determined by reward account."
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "name",
                        "type": {
                            "array": [
                                "u8",
                                14
                            ]
                        }
                    },
                    {
                        "name": "rewardPoolBitmap",
                        "type": "u8"
                    },
                    {
                        "name": "tokenAllocatedAmount",
                        "type": {
                            "defined": {
                                "name": "tokenAllocatedAmount"
                            }
                        }
                    },
                    {
                        "name": "contribution",
                        "type": "u128"
                    },
                    {
                        "name": "initialSlot",
                        "type": "u64"
                    },
                    {
                        "name": "updatedSlot",
                        "type": "u64"
                    },
                    {
                        "name": "closedSlot",
                        "type": "u64"
                    },
                    {
                        "name": "holderId",
                        "type": "u8"
                    },
                    {
                        "name": "numRewardSettlements",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                6
                            ]
                        }
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u64",
                                32
                            ]
                        }
                    },
                    {
                        "name": "rewardSettlements1",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "rewardSettlement"
                                    }
                                },
                                16
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "rewardPoolHolder",
            "docs": [
                "Reward pool holder type."
            ],
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "id",
                        "docs": [
                            "ID is determined by reward account."
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "name",
                        "type": {
                            "array": [
                                "u8",
                                14
                            ]
                        }
                    },
                    {
                        "name": "description",
                        "type": {
                            "array": [
                                "u8",
                                128
                            ]
                        }
                    },
                    {
                        "name": "numPubkeys",
                        "type": "u8"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u64",
                                32
                            ]
                        }
                    },
                    {
                        "name": "pubkeys1",
                        "docs": [
                            "List of allowed pubkeys for this holder."
                        ],
                        "type": {
                            "array": [
                                "pubkey",
                                8
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "rewardSettlement",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "rewardId",
                        "type": "u16"
                    },
                    {
                        "name": "rewardPoolId",
                        "type": "u8"
                    },
                    {
                        "name": "numSettlementBlocks",
                        "type": "u8"
                    },
                    {
                        "name": "settlementBlocksHead",
                        "type": "u8"
                    },
                    {
                        "name": "settlementBlocksTail",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                2
                            ]
                        }
                    },
                    {
                        "name": "remainingAmount",
                        "docs": [
                            "Leftovers from each settlement block when clearing"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "claimedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "claimedAmountUpdatedSlot",
                        "type": "u64"
                    },
                    {
                        "name": "settledAmount",
                        "type": "u64"
                    },
                    {
                        "name": "settlementBlocksLastSlot",
                        "type": "u64"
                    },
                    {
                        "name": "settlementBlocksLastRewardPoolContribution",
                        "type": "u128"
                    },
                    {
                        "name": "settlementBlocks",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "rewardSettlementBlock"
                                    }
                                },
                                64
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "rewardSettlementBlock",
            "docs": [
                "Exact settlement block range: [`starting_slot`, `ending_slot`)"
            ],
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "amount",
                        "type": "u64"
                    },
                    {
                        "name": "startingSlot",
                        "type": "u64"
                    },
                    {
                        "name": "startingRewardPoolContribution",
                        "type": "u128"
                    },
                    {
                        "name": "endingRewardPoolContribution",
                        "type": "u128"
                    },
                    {
                        "name": "endingSlot",
                        "type": "u64"
                    },
                    {
                        "name": "userSettledAmount",
                        "type": "u64"
                    },
                    {
                        "name": "userSettledContribution",
                        "type": "u128"
                    }
                ]
            }
        },
        {
            "name": "rewardType",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "point",
                        "fields": [
                            {
                                "name": "decimals",
                                "type": "u8"
                            }
                        ]
                    },
                    {
                        "name": "token",
                        "fields": [
                            {
                                "name": "mint",
                                "type": "pubkey"
                            },
                            {
                                "name": "program",
                                "type": "pubkey"
                            },
                            {
                                "name": "decimals",
                                "type": "u8"
                            }
                        ]
                    },
                    {
                        "name": "sol"
                    }
                ]
            }
        },
        {
            "name": "splWithdrawStakeItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "validatorStakeAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundStakeAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundStakeAccountSignerSeeds",
                        "type": {
                            "vec": "bytes"
                        }
                    },
                    {
                        "name": "tokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "stakeSolCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "stakeSolCommandState"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "stakeSolCommandItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "tokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "allocatedSolAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "stakeSolCommandResult",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "tokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "stakedSolAmount",
                        "type": "u64"
                    },
                    {
                        "name": "deductedSolFeeAmount",
                        "type": "u64"
                    },
                    {
                        "name": "mintedTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "operationReservedTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "stakeSolCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "new"
                    },
                    {
                        "name": "prepare",
                        "fields": [
                            {
                                "name": "items",
                                "type": {
                                    "vec": {
                                        "defined": {
                                            "name": "stakeSolCommandItem"
                                        }
                                    }
                                }
                            }
                        ]
                    },
                    {
                        "name": "execute",
                        "fields": [
                            {
                                "name": "items",
                                "type": {
                                    "vec": {
                                        "defined": {
                                            "name": "stakeSolCommandItem"
                                        }
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "supportedToken",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "mint",
                        "type": "pubkey"
                    },
                    {
                        "name": "program",
                        "type": "pubkey"
                    },
                    {
                        "name": "decimals",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    },
                    {
                        "name": "pricingSource",
                        "type": {
                            "defined": {
                                "name": "tokenPricingSourcePod"
                            }
                        }
                    },
                    {
                        "name": "oneTokenAsSol",
                        "docs": [
                            "informative"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "token",
                        "docs": [
                            "token deposit & withdrawal"
                        ],
                        "type": {
                            "defined": {
                                "name": "assetState"
                            }
                        }
                    },
                    {
                        "name": "rebalancingAmount",
                        "docs": [
                            "configuration: the amount requested to be unstaked as soon as possible regardless of current state, this value should be decreased by each unstaking requested amount."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "solAllocationWeight",
                        "docs": [
                            "configuration: used for staking allocation strategy."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "solAllocationCapacityAmount",
                        "type": "u64"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                64
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "tokenAllocatedAmount",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "totalAmount",
                        "type": "u64"
                    },
                    {
                        "name": "numRecords",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    },
                    {
                        "name": "records",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "tokenAllocatedAmountRecord"
                                    }
                                },
                                10
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "tokenAllocatedAmountRecord",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "amount",
                        "type": "u64"
                    },
                    {
                        "name": "contributionAccrualRate",
                        "docs": [
                            "Contribution accrual rate per 1 lamports (decimals = 2)",
                            "e.g., rate = 135 => actual rate = 1.35"
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "tokenPricingSource",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "splStakePool",
                        "fields": [
                            {
                                "name": "address",
                                "type": "pubkey"
                            }
                        ]
                    },
                    {
                        "name": "marinadeStakePool",
                        "fields": [
                            {
                                "name": "address",
                                "type": "pubkey"
                            }
                        ]
                    },
                    {
                        "name": "jitoRestakingVault",
                        "fields": [
                            {
                                "name": "address",
                                "type": "pubkey"
                            }
                        ]
                    },
                    {
                        "name": "fragmetricNormalizedTokenPool",
                        "fields": [
                            {
                                "name": "address",
                                "type": "pubkey"
                            }
                        ]
                    },
                    {
                        "name": "fragmetricRestakingFund",
                        "fields": [
                            {
                                "name": "address",
                                "type": "pubkey"
                            }
                        ]
                    },
                    {
                        "name": "orcaDexLiquidityPool",
                        "fields": [
                            {
                                "name": "address",
                                "type": "pubkey"
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "tokenPricingSourcePod",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "discriminant",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                7
                            ]
                        }
                    },
                    {
                        "name": "address",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "tokenValue",
            "docs": [
                "a value representing total asset value of a pricing source."
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "numerator",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "asset"
                                }
                            }
                        }
                    },
                    {
                        "name": "denominator",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "tokenValuePod",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "numerator",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "assetPod"
                                    }
                                },
                                33
                            ]
                        }
                    },
                    {
                        "name": "numNumerator",
                        "type": "u64"
                    },
                    {
                        "name": "denominator",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "undelegateVstCommand",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "undelegateVstCommandResult",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "unrestakeVrtCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "items",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "unrestakeVstCommandItem"
                                }
                            }
                        }
                    },
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "unrestakeVrtCommandState"
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "unrestakeVrtCommandResult",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "unrestakeVrtCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "init"
                    },
                    {
                        "name": "readVaultState"
                    },
                    {
                        "name": "unstake",
                        "fields": [
                            {
                                "vec": "bytes"
                            }
                        ]
                    }
                ]
            }
        },
        {
            "name": "unrestakeVstCommandItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "vaultAddress",
                        "type": "pubkey"
                    },
                    {
                        "name": "solAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "unstakeLstCommand",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "items",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "unstakeLstCommandItem"
                                }
                            }
                        }
                    },
                    {
                        "name": "state",
                        "type": {
                            "defined": {
                                "name": "unstakeLstCommandState"
                            }
                        }
                    },
                    {
                        "name": "splWithdrawStakeItems",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "splWithdrawStakeItem"
                                }
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "unstakeLstCommandItem",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "mint",
                        "type": "pubkey"
                    },
                    {
                        "name": "tokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "unstakeLstCommandResult",
            "type": {
                "kind": "struct",
                "fields": []
            }
        },
        {
            "name": "unstakeLstCommandState",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "init"
                    },
                    {
                        "name": "readPoolState"
                    },
                    {
                        "name": "getAvailableUnstakeAccount"
                    },
                    {
                        "name": "unstake"
                    },
                    {
                        "name": "requestUnstake"
                    }
                ]
            }
        },
        {
            "name": "userCanceledWithdrawalRequestFromFund",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "updatedUserRewardAccounts",
                        "type": {
                            "vec": "pubkey"
                        }
                    },
                    {
                        "name": "user",
                        "type": "pubkey"
                    },
                    {
                        "name": "userReceiptTokenAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "userFundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "batchId",
                        "type": "u64"
                    },
                    {
                        "name": "requestId",
                        "type": "u64"
                    },
                    {
                        "name": "requestedReceiptTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "userCreatedOrUpdatedFundAccount",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "userFundAccount",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "userCreatedOrUpdatedRewardAccount",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "userRewardAccount",
                        "type": "pubkey"
                    }
                ]
            }
        },
        {
            "name": "userDepositedToFund",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "updatedUserRewardAccounts",
                        "type": {
                            "vec": "pubkey"
                        }
                    },
                    {
                        "name": "user",
                        "type": "pubkey"
                    },
                    {
                        "name": "userReceiptTokenAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "userFundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "userSupportedTokenAccount",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "walletProvider",
                        "type": {
                            "option": "string"
                        }
                    },
                    {
                        "name": "contributionAccrualRate",
                        "type": {
                            "option": "u8"
                        }
                    },
                    {
                        "name": "depositedAmount",
                        "type": "u64"
                    },
                    {
                        "name": "mintedReceiptTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "userFundAccount",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "dataVersion",
                        "type": "u16"
                    },
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "user",
                        "type": "pubkey"
                    },
                    {
                        "name": "receiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                32
                            ]
                        }
                    },
                    {
                        "name": "withdrawalRequests",
                        "type": {
                            "vec": {
                                "defined": {
                                    "name": "withdrawalRequest"
                                }
                            }
                        }
                    }
                ]
            }
        },
        {
            "name": "userRequestedWithdrawalFromFund",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "updatedUserRewardAccounts",
                        "type": {
                            "vec": "pubkey"
                        }
                    },
                    {
                        "name": "user",
                        "type": "pubkey"
                    },
                    {
                        "name": "userReceiptTokenAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "userFundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "batchId",
                        "type": "u64"
                    },
                    {
                        "name": "requestId",
                        "type": "u64"
                    },
                    {
                        "name": "requestedReceiptTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "userRewardAccount",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "dataVersion",
                        "type": "u16"
                    },
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "user",
                        "type": "pubkey"
                    },
                    {
                        "name": "numUserRewardPools",
                        "type": "u8"
                    },
                    {
                        "name": "maxUserRewardPools",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                11
                            ]
                        }
                    },
                    {
                        "name": "userRewardPools1",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "userRewardPool"
                                    }
                                },
                                4
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "userRewardPool",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "tokenAllocatedAmount",
                        "type": {
                            "defined": {
                                "name": "tokenAllocatedAmount"
                            }
                        }
                    },
                    {
                        "name": "contribution",
                        "type": "u128"
                    },
                    {
                        "name": "updatedSlot",
                        "type": "u64"
                    },
                    {
                        "name": "rewardPoolId",
                        "type": "u8"
                    },
                    {
                        "name": "numRewardSettlements",
                        "type": "u8"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                6
                            ]
                        }
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u64",
                                8
                            ]
                        }
                    },
                    {
                        "name": "rewardSettlements1",
                        "type": {
                            "array": [
                                {
                                    "defined": {
                                        "name": "userRewardSettlement"
                                    }
                                },
                                16
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "userRewardSettlement",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "rewardId",
                        "type": "u16"
                    },
                    {
                        "name": "padding",
                        "type": {
                            "array": [
                                "u8",
                                6
                            ]
                        }
                    },
                    {
                        "name": "settledAmount",
                        "type": "u64"
                    },
                    {
                        "name": "settledContribution",
                        "type": "u128"
                    },
                    {
                        "name": "settledSlot",
                        "type": "u64"
                    },
                    {
                        "name": "claimedAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "userTransferredReceiptToken",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "updatedUserRewardAccounts",
                        "type": {
                            "vec": "pubkey"
                        }
                    },
                    {
                        "name": "source",
                        "type": "pubkey"
                    },
                    {
                        "name": "sourceReceiptTokenAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "sourceFundAccount",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "destination",
                        "type": "pubkey"
                    },
                    {
                        "name": "destinationReceiptTokenAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "destinationFundAccount",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "transferredReceiptTokenAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "userUpdatedRewardPool",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "updatedUserRewardAccounts",
                        "type": {
                            "vec": "pubkey"
                        }
                    }
                ]
            }
        },
        {
            "name": "userWithdrewFromFund",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "receiptTokenMint",
                        "type": "pubkey"
                    },
                    {
                        "name": "fundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "user",
                        "type": "pubkey"
                    },
                    {
                        "name": "userReceiptTokenAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "userFundAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "userSupportedTokenAccount",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "fundWithdrawalBatchAccount",
                        "type": "pubkey"
                    },
                    {
                        "name": "batchId",
                        "type": "u64"
                    },
                    {
                        "name": "requestId",
                        "type": "u64"
                    },
                    {
                        "name": "burntReceiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "returnedReceiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "withdrawnAmount",
                        "type": "u64"
                    },
                    {
                        "name": "deductedFeeAmount",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "withdrawalBatch",
            "serialization": "bytemuck",
            "repr": {
                "kind": "c"
            },
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "batchId",
                        "type": "u64"
                    },
                    {
                        "name": "numRequests",
                        "type": "u64"
                    },
                    {
                        "name": "receiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "enqueuedAt",
                        "type": "i64"
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                32
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "withdrawalRequest",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "batchId",
                        "type": "u64"
                    },
                    {
                        "name": "requestId",
                        "type": "u64"
                    },
                    {
                        "name": "receiptTokenAmount",
                        "type": "u64"
                    },
                    {
                        "name": "createdAt",
                        "type": "i64"
                    },
                    {
                        "name": "supportedTokenMint",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "supportedTokenProgram",
                        "type": {
                            "option": "pubkey"
                        }
                    },
                    {
                        "name": "reserved",
                        "type": {
                            "array": [
                                "u8",
                                14
                            ]
                        }
                    }
                ]
            }
        }
    ],
    "constants": [
        {
            "name": "adminPubkey",
            "type": "pubkey",
            "value": "fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby"
        },
        {
            "name": "devnetBsolMintAddress",
            "type": "pubkey",
            "value": "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1"
        },
        {
            "name": "devnetBsolStakePoolAddress",
            "type": "pubkey",
            "value": "azFVdHtAJN8BX3sbGAYkXvtdjdrT5U6rj9rovvUFos9"
        },
        {
            "name": "devnetMsolMintAddress",
            "type": "pubkey",
            "value": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"
        },
        {
            "name": "devnetMsolStakePoolAddress",
            "type": "pubkey",
            "value": "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC"
        },
        {
            "name": "devnetNsolMintAddress",
            "type": "pubkey",
            "value": "nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e"
        },
        {
            "name": "devnetProgramId",
            "type": "pubkey",
            "value": "frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ"
        },
        {
            "name": "fragsolAddressLookupTableAddress",
            "type": "pubkey",
            "value": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc"
        },
        {
            "name": "fragsolJitoVaultAccountAddress",
            "type": "pubkey",
            "value": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S"
        },
        {
            "name": "fragsolJitoVaultReceiptTokenMintAddress",
            "type": "pubkey",
            "value": "CkXLPfDG3cDawtUvnztq99HdGoQWhJceBZxqKYL2TUrg"
        },
        {
            "name": "fragsolMintAddress",
            "type": "pubkey",
            "value": "FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo"
        },
        {
            "name": "fragsolNormalizedTokenMintAddress",
            "type": "pubkey",
            "value": "nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e"
        },
        {
            "name": "fundAccountCurrentVersion",
            "docs": [
                "## Version History",
                "* v15: migrate to new layout including new fields using bytemuck. (150312 ~= 147KB)"
            ],
            "type": "u16",
            "value": "15"
        },
        {
            "name": "fundAccountOperationCommandExpirationSeconds",
            "type": "i64",
            "value": "600"
        },
        {
            "name": "fundManagerPubkey",
            "type": "pubkey",
            "value": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"
        },
        {
            "name": "jitoVaultConfigAddress",
            "type": "pubkey",
            "value": "UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3"
        },
        {
            "name": "jitoVaultProgramFeeWallet",
            "type": "pubkey",
            "value": "5eosrve6LktMZgVNszYzebgmmC7BjLK8NoWyRQtcmGTF"
        },
        {
            "name": "jitoVaultProgramId",
            "type": "pubkey",
            "value": "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8"
        },
        {
            "name": "mainnetBnsolMintAddress",
            "type": "pubkey",
            "value": "BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85"
        },
        {
            "name": "mainnetBnsolStakePoolAddress",
            "type": "pubkey",
            "value": "Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r"
        },
        {
            "name": "mainnetBsolMintAddress",
            "docs": [
                "Below address are needed to be passed to transactions which includes pricing of tokens (token deposit, withdrawal request)\nA complete list will be provided to client via address lookup table later.\n*"
            ],
            "type": "pubkey",
            "value": "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1"
        },
        {
            "name": "mainnetBsolStakePoolAddress",
            "type": "pubkey",
            "value": "stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi"
        },
        {
            "name": "mainnetJitosolMintAddress",
            "type": "pubkey",
            "value": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"
        },
        {
            "name": "mainnetJitosolStakePoolAddress",
            "type": "pubkey",
            "value": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb"
        },
        {
            "name": "mainnetMsolMintAddress",
            "type": "pubkey",
            "value": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"
        },
        {
            "name": "mainnetMsolStakePoolAddress",
            "type": "pubkey",
            "value": "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC"
        },
        {
            "name": "mainnetNsolMintAddress",
            "type": "pubkey",
            "value": "nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e"
        },
        {
            "name": "mainnetProgramId",
            "type": "pubkey",
            "value": "fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3"
        },
        {
            "name": "normalizedTokenPoolAccountCurrentVersion",
            "docs": [
                "## Version History",
                "* v1: Initial Version",
                "* v2: Add `normalized_token_decimals`, .., `one_normalized_token_as_sol` fields"
            ],
            "type": "u16",
            "value": "2"
        },
        {
            "name": "normalizedTokenWithdrawalAccountCurrentVersion",
            "docs": [
                "## Version History",
                "* v1: Initial Version"
            ],
            "type": "u16",
            "value": "1"
        },
        {
            "name": "programRevenueAddress",
            "type": "pubkey",
            "value": "XEhpR3UauMkARQ8ztwaU9Kbv16jEpBbXs9ftELka9wj"
        },
        {
            "name": "rewardAccountCurrentVersion",
            "docs": [
                "## Version History",
                "* v34: Initial Version (Data Size = 342064 ~= 335KB)"
            ],
            "type": "u16",
            "value": "34"
        },
        {
            "name": "target",
            "type": "string",
            "value": "\"mainnet\""
        },
        {
            "name": "userRewardAccountCurrentVersion",
            "docs": [
                "## Version History",
                "* v_1: Initial Version"
            ],
            "type": "u16",
            "value": "1"
        }
    ]
};
