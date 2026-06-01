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
