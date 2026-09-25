import messages from './zh-CN.json';
const folded = Object.fromEntries(Object.entries(messages).map(([key, value]) => [key.toLocaleLowerCase('en-US'), value]));

/** Translate interface copy only. Never apply this to user-authored memory or screenplay text. */
export function zh(message: string): string {
    return (messages as Record<string, string>)[message] ?? folded[message.toLocaleLowerCase('en-US')] ?? message;
}

export const UI_LOCALE = 'zh-CN';
export function localDate(value: string | number | Date, options?: Intl.DateTimeFormatOptions): string {
    return new Date(value).toLocaleString(UI_LOCALE, options);
}
