export abstract class EnumFlags<E extends number> {
    protected abstract get all(): E[];
    protected abstract get names(): Record<E, string>;
    public bits: number;

    constructor(bits: number) {
        this.bits = bits;
    }

    /** Returns true if a singular scope is contained in this instance */
    public has(flag: E): boolean {
        return (this.bits & flag) === flag;
    }

    /** Add a new flag to the instance */
    public add(flag: E): this {
        this.bits |= flag;
        return this;
    }

    /** Remove a flag from the instance */
    public remove(flag: E): this {
        this.bits &= ~flag;
        return this;
    }

    /** Returns a Scopes instance with all available scopes */
    public allSet(): this {
        let allFlags = 0;
        for (const flag of this.all) {
            allFlags |= flag;
        }

        return this.add(allFlags as E);
    }

    public get isEmpty(): boolean {
        return this.bits === 0;
    }

    /** Returns an array containing all the **set** scopes in the enum form */
    public get enumArray(): E[] {
        return this.all.filter((v) => this.has(v));
    }

    /** Returns an array containing all the **set** scopes in strings *(human readable form)* */
    public toStrings(): string[] {
        let array: string[] = [];
        let current_flags = this.enumArray;

        for (let flag of current_flags) {
            array.push(this.names[flag]);
        }

        return array;
    }
}
