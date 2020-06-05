export default async function (name, pkgDir='./pkg') {
    const pkgJs = await (await fetch(`${pkgDir}/${name}.js`)).text();

    // Create the 'pure' version of the wasm_bindgen's `init()`
    const initJs = `return () => { ${pkgJs} return wasm_bindgen; };`;
    const init = (new Function(initJs)).call(null);

    const wbg = init();
    const wasm = await wbg(`${pkgDir}/${name}_bg.wasm`);
    // console.log('wasm:', wasm);
    wbg.app(pkgDir);

    return wbg;
}
