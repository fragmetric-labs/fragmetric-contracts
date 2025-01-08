import {LRUCache} from "lru-cache";
import {ICache, ICacheFactory} from "./cache";

export const Cache: ICacheFactory = {
    create: ({ ttlSeconds, keyPrefix } = {}) => {
        return new LocalStorageCache(ttlSeconds, keyPrefix);
    }
}

type LocalStorageCacheEntry<T extends {}> = {
    v: T; // value
    e: number; // expiry time in ms
}

export class LocalStorageCache<T extends {}> implements ICache<T> {
    private readonly keyPrefix: string;
    private readonly cache: LRUCache<string, T>;

    constructor(ttlSeconds = 600, keyPrefix = 'default') {
        this.cache = new LRUCache({
            max: 1000,
            ttl: 1000 * Math.max(ttlSeconds, 1),
            ttlAutopurge: false,
            allowStale: false,
        });
        this.keyPrefix = keyPrefix;
    }

    private getLocalStorageKey(key: string): string {
        return `fragmetric:${this.keyPrefix}:${key}`;
    }

    public get(key: string): T | undefined {
        if (this.cache.has(key)) {
            return this.cache.get(key);
        }

        const localStorageKey = this.getLocalStorageKey(key);
        const stored = localStorage.getItem(localStorageKey);
        if (stored) {
            try {
                const { v, e } = JSON.parse(stored) as LocalStorageCacheEntry<T>;
                const now = Date.now();
                const ttl = e - now;
                if (ttl > 0) {
                    this.cache.set(key, v, { ttl });
                    return v;
                }
                localStorage.removeItem(localStorageKey);
            } catch (err) {
                console.error(err);
                localStorage.removeItem(localStorageKey)
            }
        }

        return undefined;
    }

    public set(key: string, value: T, ttlSeconds?: number): T {
        const ttl = ttlSeconds ? Math.max(ttlSeconds, 1) * 1000 : this.cache.ttl;

        this.cache.set(key, value, { ttl });

        const entry: LocalStorageCacheEntry<T> = { v: value, e: Date.now() + ttl };
        localStorage.setItem(this.getLocalStorageKey(key), JSON.stringify(entry));

        return value;
    }

    public has(key: string) {
        if (this.cache.has(key)) return true;

        return this.get(key) !== undefined;
    }

    public delete(key: string): void {
        this.cache.delete(key);
        localStorage.removeItem(this.getLocalStorageKey(key));
    }

    public clear(): void {
        this.cache.clear();

        const localStorageKeyPrefix = this.getLocalStorageKey('');
        for (const key in Object.keys(localStorage)) {
            if (key.startsWith(localStorageKeyPrefix)) {
                localStorage.removeItem(key);
            }
        }
    }
}
