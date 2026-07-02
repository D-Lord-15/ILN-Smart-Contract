/**
 * TokenRegistry — centralised multi-token metadata for the ILN SDK (issue #227).
 *
 * Provides contract addresses, decimal precision, and minimum amounts
 * for supported tokens on Stellar testnet and mainnet.
 */

export type NetworkName = "testnet" | "mainnet";

export interface TokenInfo {
  /** Token symbol (e.g. "USDC"). */
  symbol: string;
  /** Stellar contract address (SAC or SEP-41) for the given network. */
  contractAddress: string;
  /** Number of decimal places used by the token (e.g. 7 for USDC on Stellar). */
  decimals: number;
  /** Minimum meaningful amount in base units (to guard against dust). */
  minimumAmount: bigint;
}

/** Built-in token registry entries per network. */
const REGISTRY: Record<NetworkName, Record<string, TokenInfo>> = {
  testnet: {
    USDC: {
      symbol: "USDC",
      contractAddress: "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA",
      decimals: 7,
      minimumAmount: 1_000_000n, // 0.1 USDC
    },
    EURC: {
      symbol: "EURC",
      contractAddress: "GB3Q6QDZYTHWT7E5PVS3W7FUT5GVAFC5KSZFFLPU25GO575XC74F3X6U",
      decimals: 7,
      minimumAmount: 1_000_000n,
    },
    XLM: {
      symbol: "XLM",
      contractAddress: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
      decimals: 7,
      minimumAmount: 10_000_000n, // 1 XLM
    },
  },
  mainnet: {
    USDC: {
      symbol: "USDC",
      contractAddress: "CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75",
      decimals: 7,
      minimumAmount: 1_000_000n,
    },
    EURC: {
      symbol: "EURC",
      contractAddress: "CDTKPWPLOURQA2SGTKTUQOWRCBZEORB4BWBOMJ3D3ZTQQSGE5F6JBQLV",
      decimals: 7,
      minimumAmount: 1_000_000n,
    },
    XLM: {
      symbol: "XLM",
      contractAddress: "CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA",
      decimals: 7,
      minimumAmount: 10_000_000n,
    },
  },
};

export class TokenRegistry {
  private readonly network: NetworkName;

  constructor(network: NetworkName = "testnet") {
    this.network = network;
  }

  /**
   * Look up a token by symbol.
   * @throws {Error} when the symbol is not found for the current network.
   */
  get(symbol: string): TokenInfo {
    const entry = REGISTRY[this.network][symbol.toUpperCase()];
    if (!entry) {
      throw new Error(
        `Token "${symbol}" is not registered for network "${this.network}". ` +
          `Supported: ${Object.keys(REGISTRY[this.network]).join(", ")}`
      );
    }
    return entry;
  }

  /** Returns all registered tokens for the current network. */
  list(): TokenInfo[] {
    return Object.values(REGISTRY[this.network]);
  }

  /**
   * Register a custom token at runtime (e.g. project-specific SAC tokens).
   * Custom entries override built-ins for the same symbol.
   */
  register(info: TokenInfo): void {
    REGISTRY[this.network][info.symbol.toUpperCase()] = info;
  }

  /**
   * Convert a human-readable amount to base units.
   * @example registry.toBaseUnits("USDC", 10.5) // 105_000_000n
   */
  toBaseUnits(symbol: string, humanAmount: number): bigint {
    const { decimals } = this.get(symbol);
    return BigInt(Math.round(humanAmount * 10 ** decimals));
  }

  /**
   * Convert base units back to a human-readable number.
   * @example registry.fromBaseUnits("USDC", 105_000_000n) // 10.5
   */
  fromBaseUnits(symbol: string, baseAmount: bigint): number {
    const { decimals } = this.get(symbol);
    return Number(baseAmount) / 10 ** decimals;
  }
}

/** Default singleton — testnet. Swap via `new TokenRegistry("mainnet")`. */
export const tokenRegistry = new TokenRegistry("testnet");
