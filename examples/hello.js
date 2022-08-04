setTimeout(() => {
    console.log("Hello world");
}, 0)

globalThis.i = globalThis.i ?? 0;
globalThis.i = globalThis.i + 1;
console.log(globalThis.i)