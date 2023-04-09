import init, { Context } from "../../debugdumpgen/out/debugdumpgen";
import { Graphviz as GraphvizLoader } from "@hpcc-js/wasm/dist/graphviz";

const includesInput = document.getElementById("includes") as HTMLInputElement;
const excludesInput = document.getElementById("excludes") as HTMLInputElement;
const scheduleSelect = document.getElementById("scheduleSelect") as HTMLSelectElement;
const svgElement = document.getElementById("svg")! as HTMLImageElement;
const shareButton = document.getElementById("share")! as HTMLButtonElement;

let schedules: string[] | undefined = undefined;

let scheduleSelectFromQuery: string | undefined = undefined;

function loadQuery() {
    const params = new URLSearchParams(window.location.search);
    let include = params.get("include");
    let exclude = params.get("exclude");
    let schedule = params.get("schedule");
    let renderApp = params.get("renderApp");

    if (include) includesInput.value = include;
    if (exclude) excludesInput.value = exclude;

    if (schedule) {
        scheduleSelectFromQuery = `${schedule}:${renderApp ?? "false"}`;
    }
}
loadQuery();

shareButton.addEventListener("click", () => {
    let [schedule, renderApp] = scheduleSelect.value.split(":");

    let query = `schedule=${schedule}`;
    if (renderApp !== "false") {
        query = `${query}&renderApp=${renderApp}`;
    }
    if (includesInput.value) {
        query = `${query}&include=${includesInput.value}`;
    }
    if (excludesInput.value) {
        query = `${query}&exclude=${excludesInput.value}`;
    }

    let url = `${window.location.protocol}//${window.location.host}${window.location.pathname}?${query}`;
    navigator.clipboard.writeText(url);

    window.history.pushState({ path: url }, '', url);
});

async function run() {
    let [graphviz, _] = await Promise.all([GraphvizLoader.load(), init()]);
    let context = new Context();


    let main_schedules = context.main_schedules();
    let non_main_schedules = context.non_main_schedules();
    let render_schedules = context.render_schedules();

    let optgroup = (label: string, options: string[], render: boolean) => {
        let group = document.createElement("optgroup");
        group.label = label;
        for (let name of options) {
            let option = document.createElement("option");
            option.value = name + ":" + render;
            option.innerText = name;
            group.appendChild(option);
        }
        return group;
    };
    scheduleSelect.replaceChildren(optgroup("Main", main_schedules, false), optgroup("Other", non_main_schedules, false), optgroup("Render", render_schedules, true));

    if (scheduleSelectFromQuery) {
        scheduleSelect.value = scheduleSelectFromQuery;
    } else {
        scheduleSelect.value = "PreUpdate:false";
    }

    let updateSvgElement = (content: string) => {
        let svg = timed("dot", () => graphviz.dot(content, "svg"));
        svgElement.innerHTML = svg;
    };

    let regenerate = () => {
        console.log(`Generate ${scheduleSelect.value}`);
        try {
            let [schedule, renderApp] = scheduleSelect.value.split(":");
            let isRenderApp = renderApp === "true";

            if (!schedule) return;

            let svg = timed("svg", () => context.generate_svg(
                schedule,
                isRenderApp,
                includesInput.value,
                excludesInput.value
            ));
            updateSvgElement(svg);
        } catch (e) {
            console.error(e);

            let msg;
            if (e instanceof Error) {
                msg = e.message;
            } else {
                msg = "" + e;
            }

            alert("Failed to generate schedule: " + msg);
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
