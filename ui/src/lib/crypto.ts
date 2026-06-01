// Transformed from https://blog.elantha.com/encrypt-in-the-browser

export type EncryptedContent = {
    content_type: string;
    content: string;
};

export const encrypt = async (content: string, password: string) => {
    const salt = crypto.getRandomValues(new Uint8Array(16));
    const key = await get_key(password, salt);
    const iv = crypto.getRandomValues(new Uint8Array(12));

    const content_bytes = new TextEncoder().encode(content);

    const cipher = new Uint8Array(
        await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, content_bytes)
    );

    return into_encrypted_string({ salt, iv, cipher });
};

export const decrypt = async (encrypted_data: string, password: string) => {
    const { salt, iv, cipher } = parse_encrypted_string(encrypted_data);
    const key = await get_key(password, salt);

    const content_bytes = new Uint8Array(
        await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, key, cipher)
    );

    return new TextDecoder().decode(content_bytes);
};

const get_key = async (password: string, salt: BufferSource) => {
    const password_bytes = new TextEncoder().encode(password);

    const initial_key = await crypto.subtle.importKey(
        'raw',
        password_bytes,
        { name: 'PBKDF2' },
        false,
        ['deriveKey']
    );

    return crypto.subtle.deriveKey(
        { name: 'PBKDF2', salt, iterations: 100000, hash: 'SHA-256' },
        initial_key,
        { name: 'AES-GCM', length: 256 },
        false,
        ['encrypt', 'decrypt']
    );
};

const parse_encrypted_string = (
    encrypted_string: string
): { salt: BufferSource; iv: BufferSource; cipher: BufferSource } => {
    const [salt, iv, cipher] = encrypted_string.split(':');

    // Using as any here because fromBase64 is relatively new.
    /* eslint-disable @typescript-eslint/no-explicit-any */
    return {
        salt: (Uint8Array as any).fromBase64(salt),
        iv: (Uint8Array as any).fromBase64(iv),
        cipher: (Uint8Array as any).fromBase64(cipher),
    };
    /* eslint-enable @typescript-eslint/no-explicit-any */
};

const into_encrypted_string = ({
    salt,
    iv,
    cipher,
}: {
    salt: Uint8Array;
    iv: Uint8Array;
    cipher: Uint8Array;
}): string => {
    // Using as any here because toBase64 is relatively new.
    /* eslint-disable @typescript-eslint/no-explicit-any */
    const s_str = (salt as any).toBase64();
    const i_str = (iv as any).toBase64();
    const c_str = (cipher as any).toBase64();
    /* eslint-enable @typescript-eslint/no-explicit-any */

    return `${s_str}:${i_str}:${c_str}`;
};
