import * as asyncCache from 'async-cache-dedupe';
import {LRUCache} from "lru-cache";

export interface ICache<T = any> {
    get(key: string): T | undefined;
    // returns the set value
    set(key: string, value: T, ttlSeconds?: number): T;
    has(key: string): boolean;
    delete(key: string): void;
    clear(): void;
}

export interface ICacheFactory<T = any> {
    create(params?: {
        ttlSeconds?: number;
    }): ICache<T>;
}

export const InMemoryCache: ICacheFactory = {
    create: ({ ttlSeconds } = {}) => {
        return new InMemoryCacheImpl(ttlSeconds);
    }
}

class InMemoryCacheImpl<T extends {} = any> implements ICache<T> {
    private readonly cache: LRUCache<string, T>;

    constructor(ttlSeconds = 600) {
        this.cache = new LRUCache({
            max: 1000,
            ttl: Math.max(ttlSeconds, 1) * 1000,
            ttlAutopurge: false,
            allowStale: false,
        });
    }

    get(key: string): T | undefined {
        return this.cache.get(key);
    }

    set(key: string, value: T, ttlSeconds?: number): T {
        const ttl = ttlSeconds ? Math.max(ttlSeconds, 1) * 1000 : this.cache.ttl;
        this.cache.set(key, value, { ttl });
        return value;
    }

    has(key: string): boolean {
        return this.cache.has(key);
    }

    delete(key: string): void {
        this.cache.delete(key);
    }

    clear(): void {
        this.cache.clear();
    }
}

const promiseCache = asyncCache.createCache({
    ttl: 0,
    stale: 0,
    // onDedupe: (key) => console.log('deduped', key),
});

let promiseCacheIndex = 0;

export function dedupe<T extends (args: any) => Promise<any>>(fn: T): T {
    const name = `fn${promiseCacheIndex++}`;
    return promiseCache.define(name, fn)[name];
}
