/** An enum with the unique purpose to be used for... making my life a half 1% more easier */
// enum variant = ACTUAL KEY FOR e.key == ???
// this might be atually uselsss but hey FOR ME its a nice to have
export enum KeysThatMatter {
    Escape = "Escape"
}

export const humanReadableKeymap: Record<KeysThatMatter, string> = {
    [KeysThatMatter.Escape]: "esc"
};

export const getHumanDisplayKey = (key: KeysThatMatter): string => {
    const data = humanReadableKeymap[key] ?? key;
    return data;
};
