type Point = {
	readonly x: number;
	readonly y: number;
}
function load(): Array<Array<Array<Point>>> {
	let storage = window.sessionStorage;
	let str = storage.getItem("ankizh_canvas_statke");
	return (str===null) ? [[]] : JSON.parse(str);
}
function save(state: Array<Array<Array<Point>>>) {
	let storage = window.sessionStorage;
	storage.setItem("ankizh_canvas_statke", JSON.stringify(state));
}
// on resize
function resetCtx(ctx: CanvasRenderingContext2D) {
	ctx.reset();
	ctx.lineWidth = 20;
	ctx.lineCap = "round";
	ctx.lineJoin = "round";
	// TODO
}
let touching: number = -1;
function redraw(canvas_i: number) {
	for(let div of [frontCanvases, backCanvases]) {
		if(div!==undefined) {
			let cel: HTMLCanvasElement = div.children[canvas_i].firstChild as HTMLCanvasElement;
			let ctx: CanvasRenderingContext2D = cel.getContext("2d")!;
			resetCtx(ctx);
			let strokes = state[canvas_i];
			for (let i in strokes) {
				if (strokes[i].length>1) {
					//TODO: set color if back
					ctx.moveTo(strokes[i][0].x*cel.width, strokes[i][0].y*cel.height);
					ctx.beginPath();
					for (let j = 1; j < strokes[i].length; j++) {
						ctx.lineTo(strokes[i][j].x*cel.width, strokes[i][j].y*cel.height);
					}
					ctx.stroke();
				}
			}
		}
	}
}
function endTouch() {
	if(touching != -1) {
		redraw(touching);
		save(state);
	}
	touching = -1;
}
// should have clientX and clientY
function continueTouch(x: number, y: number) {
	if(touching != -1 && frontCanvases !== undefined) {
		let cel: HTMLCanvasElement = frontCanvases.children[touching].firstChild as HTMLCanvasElement;
		let br = cel.getBoundingClientRect();
		if(x >= br.left && x < br.right && y >= br.top && y < br.bottom) {
			let ctx: CanvasRenderingContext2D = cel.getContext("2d")!;
			let stroke = state[touching][state[touching].length-1];

			x = (x-br.left)/br.width;
			y = (y-br.top)/br.height;

			let lx = stroke[stroke.length-1].x;
			let ly = stroke[stroke.length-1].y;
			if(Math.abs(x-lx)<0.005 && Math.abs(y-ly)<0.005) {
				return;
			}

			ctx.lineTo(x*cel.width,y*cel.height);
			ctx.stroke();
			stroke.push({x:x,y:y});
		} else {
			endTouch();
		}
	}
}
// should have clientX and clientY
function startTouch(x: number, y: number) {
	if (touching != -1) {
		endTouch();
	}
	if(frontCanvases!==undefined) {
		for(let i = 0; i < frontCanvases.children.length; i++) {
			let cel: HTMLCanvasElement = frontCanvases.children[i].firstChild as HTMLCanvasElement;
			let br = cel.getBoundingClientRect();
			if(x >= br.left && x < br.right && y >= br.top && y < br.bottom) {
				let ctx: CanvasRenderingContext2D = cel.getContext("2d")!;
				x=(x-br.left)/br.width;
				y=(y-br.top)/br.height;
				touching = i;
				ctx.moveTo(x*cel.width,y*cel.height);
				ctx.beginPath();
				state[i].push([{x:x,y:y}]);
				break;
			}
		}
	}
}

let state: Array<Array<Array<Point>>> = load();
let frontDiv: HTMLElement|undefined = document.getElementById("ac-front")??undefined;
let backDiv: HTMLElement|undefined = document.getElementById("ac-back")??undefined;

function add_canvas() {
	const CSIZE: number = 1024;
	if(frontDiv!==undefined) {
		let div = document.createElement("span");
		div.className="ac-front-canvas-span";

		let canvas = document.createElement("canvas");
		canvas.height = CSIZE;
		canvas.width = CSIZE;
		let ctx: CanvasRenderingContext2D = canvas.getContext("2d")!;
		resetCtx(ctx);
		canvas.className = "ac-front-canvas";

		let idx = frontDiv.children[0].children.length;
		
		let undoButton = document.createElement("button");
		let undoButtonText = document.createTextNode("←");
		undoButton.appendChild(undoButtonText);
		undoButton.addEventListener("click", () => {
			if(state[idx].length > 0 ){
				state[idx].pop();
				redraw(idx);
			}
		})

		div.appendChild(canvas);
		div.appendChild(undoButton);

		frontDiv.children[0].appendChild(div);
	}
	if(backDiv!==undefined) {
		let div = document.createElement("span");
		let canvas = document.createElement("canvas");
		canvas.height = CSIZE;
		canvas.width = CSIZE;
		let ctx: CanvasRenderingContext2D = canvas.getContext("2d")!;
		resetCtx(ctx);
		canvas.className = "ac-back-canvas";
		div.appendChild(canvas);
		backDiv.children[0].appendChild(div);
	}
}
	

// setup front
if(frontDiv !== undefined && frontDiv.children.length == 0) {
	let canvasDiv = document.createElement("span");
	canvasDiv.className = "ac-front-canvasdiv";

	canvasDiv.addEventListener("mousedown", (ev: MouseEvent) => {ev.preventDefault();startTouch(ev.clientX, ev.clientY);});
	canvasDiv.addEventListener("touchstart", (ev: TouchEvent) => {ev.preventDefault();startTouch(ev.targetTouches[0].clientX, ev.targetTouches[0].clientY);});
	canvasDiv.addEventListener("mousemove", (ev: MouseEvent) => {ev.preventDefault();continueTouch(ev.clientX, ev.clientY);});
	canvasDiv.addEventListener("touchmove", (ev: TouchEvent) => {ev.preventDefault();continueTouch(ev.targetTouches[0].clientX, ev.targetTouches[0].clientY);});
	canvasDiv.addEventListener("mouseup", (ev: MouseEvent) => {ev.preventDefault();endTouch()});
	canvasDiv.addEventListener("touchend", (ev: TouchEvent) => {ev.preventDefault();endTouch()});
	canvasDiv.addEventListener("mouseleave", (ev: MouseEvent) => {ev.preventDefault();endTouch()});
	canvasDiv.addEventListener("touchcancel", (ev: TouchEvent) => {ev.preventDefault();endTouch()});

	let controlsDiv = document.createElement("span");
	controlsDiv.className = "ac-front-controls";

	let removeButton = document.createElement("button");
	let removeButtonText = document.createTextNode("-");
	removeButton.appendChild(removeButtonText);
	removeButton.addEventListener("click", () => {
		if(state.length>1) {
			if(frontDiv!==undefined) {
				frontDiv.children[0].removeChild(frontDiv.children[0].lastChild!);
			}
			if(backDiv!==undefined) {
				backDiv.children[0].removeChild(backDiv.children[0].lastChild!);
			}
			state.pop();
		}
	});

	let addButton = document.createElement("button");
	let addButtonText = document.createTextNode("+");
	addButton.appendChild(addButtonText);
	addButton.addEventListener("click", () => {
		state.push([]);
		add_canvas();
	});

	controlsDiv.appendChild(removeButton);
	controlsDiv.appendChild(addButton);

	// TODO: click events

	frontDiv.appendChild(canvasDiv);
	frontDiv.appendChild(controlsDiv);
}
// setup back
if(backDiv !== undefined && backDiv.children.length == 0) {
	let canvasDiv = document.createElement("div");
	canvasDiv.className = "ac-back-canvasdiv";
	backDiv.appendChild(canvasDiv);
}
// add canvases
for(let i=0; i < state.length; i++) {
	add_canvas();
}
let frontCanvases: Element|undefined = frontDiv?.children[0];
let backCanvases: Element|undefined = backDiv?.children[0];
for(let i=0; i < state.length; i++) {
	redraw(i);
}


