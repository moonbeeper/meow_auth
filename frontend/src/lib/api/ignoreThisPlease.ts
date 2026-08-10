import { API_URL } from "$app/env/public";

export const apiUrlForOrval = API_URL;

export const isOk = <T extends { status: number }>(res: T): res is T & { status: 200 } =>
    res.status >= 200 && res.status < 300;
