import svgPanZoom from "svg-pan-zoom";
// import init, { Context } from "../../debugdumpgen/out/debugdumpgen";
import init, { Context } from "../../target/release/wasm-bindgen/debugdumpgen";
import { Graphviz } from "@hpcc-js/wasm-graphviz";

const includesInput = document.getElementById("includes") as HTMLInputElement;
const excludesInput = document.getElementById("excludes") as HTMLInputElement;
const scheduleSelect = document.getElementById("scheduleSelect") as HTMLSelectElement;
const svgElement = document.getElementById("svgContainer")! as HTMLImageElement;
const shareButton = document.getElementById("share")! as HTMLButtonElement;
const openInNewTabButton = document.getElementById("openInNewTab")! as HTMLButtonElement;


let lastGeneratedSvg: string | null = null;

function registerScroll(svg: SVGGraphicsElement) {
    svgPanZoom(svg, {
        controlIconsEnabled: true,
        zoomScaleSensitivity: 0.3,
    });

    /*svg.onwheel = function (event) {
        event.preventDefault();

        // set the scaling factor (and make sure it's at least 10%)
        let scale = event.deltaY / 1000;
        scale = Math.abs(scale) < .1 ? .1 * event.deltaY / Math.abs(event.deltaY) : scale;

        // get point in SVG space
        let pt = new DOMPoint(event.clientX, event.clientY);
        pt = pt.matrixTransform(svg.getScreenCTM()?.inverse());

        // get viewbox transform
        let [x, y, width, height] = svg.getAttribute('viewBox')!.split(' ').map(Number);

        // get pt.x as a proportion of width and pt.y as proportion of height
        let [xPropW, yPropH] = [(pt.x - x) / width, (pt.y - y) / height];

        // calc new width and height, new x2, y2 (using proportions and new width and height)
        let [width2, height2] = [width + width * scale, height + height * scale];
        let x2 = pt.x - xPropW * width2;
        let y2 = pt.y - yPropH * height2;

        svg.setAttribute('viewBox', `${x2} ${y2} ${width2} ${height2}`);
    };*/
}

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

openInNewTabButton.addEventListener("click", () => {
    if (!lastGeneratedSvg) return;

    let blob = new Blob([lastGeneratedSvg], { type: "image/svg+xml" });
    let url = URL.createObjectURL(blob);
    let win = open(url);

    if (win) {
        win.onload = () => URL.revokeObjectURL(url);
    } else {
        URL.revokeObjectURL(url);
    }
});

async function run() {
    let [graphviz, _] = await Promise.all([Graphviz.load(), init()]);
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

        lastGeneratedSvg = new XMLSerializer().serializeToString(svgElement);

        registerScroll(svgElement.querySelector("svg") as SVGGraphicsElement);
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
