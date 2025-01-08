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
        keyPrefix?: string;
        ttlSeconds?: number;
    }): ICache<T>;
}

// will be aliased to './cache_impl.browser' in browser bundle
export { Cache } from './cache_impl';
