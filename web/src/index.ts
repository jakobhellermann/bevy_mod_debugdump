import init, { Context } from "../../debugdumpgen/out/debugdumpgen";
import { Graphviz as GraphvizLoader } from "@hpcc-js/wasm/dist/graphviz";

const includesInput = document.getElementById("includes") as HTMLInputElement;
const excludesInput = document.getElementById("excludes") as HTMLInputElement;
const scheduleSelect = document.getElementById("scheduleSelect") as HTMLSelectElement;
const svgElement = document.getElementById("svg")! as HTMLImageElement;
const shareButton = document.getElementById("share")! as HTMLButtonElement;

function loadQuery() {
    const params = new URLSearchParams(window.location.search);
    let include = params.get("include");
    let exclude = params.get("exclude");
    let schedule = params.get("schedule");

    if (include) includesInput.value = include;
    if (exclude) excludesInput.value = exclude;
    if (schedule && ["Main", "Startup", "RenderExtract", "RenderMain"].includes(schedule)) scheduleSelect.value = schedule;
}
loadQuery();

shareButton.addEventListener("click", () => {
    let query = `schedule=${scheduleSelect.value}&include=${includesInput.value}&exclude=${excludesInput.value}`;
    let url = `${window.location.protocol}//${window.location.host}${window.location.pathname}?${query}`;
    navigator.clipboard.writeText(url);

    window.history.pushState({ path: url }, '', url);
});

async function run() {
    let [graphviz, _] = await Promise.all([GraphvizLoader.load(), init()]);
    let context = new Context();

    let updateSvgElement = (content: string) => {
        let svg = timed("dot", () => graphviz.dot(content, "svg"));
        svgElement.innerHTML = svg;
    };

    let regenerate = () => {
        console.log(`Generate ${scheduleSelect.value}`);
        try {
            let svg = timed("svg", () => context.generate_svg(
                scheduleSelect.value,
                includesInput.value,
                excludesInput.value
            ));
            updateSvgElement(svg);
        } catch (e) {
            console.error(e);
            alert(e);
        }

    };

    scheduleSelect.addEventListener("change", regenerate);
    includesInput.addEventListener("input", regenerate);
    excludesInput.addEventListener("input", regenerate);
    regenerate();
}

function timed<T>(name: string, f: () => T): T {
    console.time(name);
    let ret = f();
    console.timeEnd(name);
    return ret;
}

run().catch(console.error);
