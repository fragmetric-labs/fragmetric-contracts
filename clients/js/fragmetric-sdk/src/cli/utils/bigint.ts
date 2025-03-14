export function createBigIntToJSONShim() {
  (BigInt.prototype as any).toJSON = function () {
    return this.toString();
  };
}
