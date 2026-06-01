import { PUBLIC_PASTA_UI_API } from "$env/static/public";
import { encrypt } from "./crypto";

export type EncryptedContent = {
    content_type: string;
    content: string;
};

export const ENCRYPTION_TYPE = 'application/aes256gcm-encrypted';

export const compress = async (data: string) => {
    const rs = new ReadableStream({
        start: (controller) => {
            controller.enqueue(new TextEncoder().encode(data));
            controller.close();
        },
    });

    const compressed = rs.pipeThrough(new CompressionStream('gzip'));

    const chunks = [];
    const reader = compressed.getReader();
    while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        chunks.push(value);
    }

    return new Blob(chunks);
};

export const save_content = async (content: string, password: string | '', content_type: string): Promise<{ status: 'ok', link: string, slug: string } | { status: 'error', message: string }> => {
    const encrypted = !!password;

    const body = encrypted
        ? await encrypt(
              JSON.stringify(<EncryptedContent>{
                  content_type,
                  content: btoa(content),
              }),
              password
          )
        : content;

    const res = await fetch(`${PUBLIC_PASTA_UI_API}/post`, {
        method: 'PUT',
        headers: {
            'content-type': encrypted ? ENCRYPTION_TYPE : content_type,
            'content-encoding': 'gzip',
        },
        body: await compress(body),
    });

    // return res;

    if (res.status === 201) {
        const slug = await res.text();
        const link = `${location.origin}/#${slug}`;
        navigator.clipboard.writeText(link);

        return {
            status: 'ok',
            link,
            slug,
        };
    } else {
        let message;
        switch (res.status) {
            case 409:
                message = 'Slug already exists';
            case 413:
                message = 'Payload too large';
            default:
                message = 'Unknown error occurred';
        }
        return {
            status: 'error',
            message,
        };
    }
};
