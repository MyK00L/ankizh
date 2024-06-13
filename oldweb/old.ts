
class Point {
	x: number=0;
	y: number=0;
	constructor(x: number, y: number) {
		this.x=x;
		this.y=y;
	}
}

class Stroke {
	points: Point[]=[];
	is_empty = (): boolean => {
		return this.points.length == 0;
	}
}

class StrokeManager {
	strokes: Stroke[]=[];
	is_empty=(): boolean=>{
		return this.strokes.length == 0;
	}
	pop=():Stroke=>{
		return this.strokes.pop()!;
	}
	push=(s: Stroke)=>{
		this.strokes.push(s);
	}
	add_point=(p: Point)=>{
		this.strokes[this.strokes.length-1].points.push(p);
	}
	add_empty_stroke=()=>{
		this.push(new Stroke());
	}
	add_stroke=(start: Point)=>{
		this.add_empty_stroke()
		this.add_point(start)	
	}
	clear=()=>{
		this.strokes=[];
	}
}

class CanvasManager {
	private sm: StrokeManager=new StrokeManager();
	private redo_stack: StrokeManager=new StrokeManager();
	private readonly canvas: HTMLCanvasElement;
	private is_mouse_down: boolean = false;
	private get ctx(): CanvasRenderingContext2D {
		return this.canvas.getContext("2d") as CanvasRenderingContext2D;
	}
	public get strokes(): StrokeManager {
		return this.sm
	}
	get_event_pos=(event: MouseEvent): Point=>{
		return new Point(event.clientX - this.canvas.offsetLeft, event.clientY - this.canvas.offsetTop);
	}
	onMouseDown=(event: MouseEvent)=>{
		this.is_mouse_down = true;
		this.start_new_stroke(this.get_event_pos(event));
	}
	onMouseUp=(_event: MouseEvent)=>{
		this.is_mouse_down = false;
	}
	onMouseMove=(event: MouseEvent)=>{
		if(this.is_mouse_down) {
			this.continue_stroke(this.get_event_pos(event));
		}
	}
	constructor(canvasId: string) {
		this.canvas=document.getElementById(canvasId) as HTMLCanvasElement;
		
		this.canvas.removeEventListener("mousedown", this.onMouseDown);		
		this.canvas.removeEventListener("mouseup", this.onMouseUp);		
		this.canvas.removeEventListener("mouseleave", this.onMouseUp);		
		this.canvas.removeEventListener("mousemove", this.onMouseMove);
		
		this.canvas.addEventListener("mousedown", this.onMouseDown);		
		this.canvas.addEventListener("mouseup", this.onMouseUp);		
		this.canvas.addEventListener("mouseleave", this.onMouseUp);		
		this.canvas.addEventListener("mousemove", this.onMouseMove);
		
		this.ctx.reset();
	}
	stop=()=>{
		this.canvas.removeEventListener("mousedown", this.onMouseDown);		
		this.canvas.removeEventListener("mouseup", this.onMouseUp);		
		this.canvas.removeEventListener("mouseleave", this.onMouseUp);		
		this.canvas.removeEventListener("mousemove", this.onMouseMove);
	}
	start_new_stroke=(start: Point)=>{
		this.redo_stack.clear();
		this.sm.add_stroke(start);
		this.ctx.beginPath();
		this.ctx.moveTo(start.x, start.y);
	}
	continue_stroke=(point: Point)=>{
		this.sm.add_point(point);
		
		this.ctx.lineTo(point.x, point.y);
		this.ctx.lineWidth = 4;
		this.ctx.lineCap = "round";
		this.ctx.stroke();
	}
	undo=()=>{
		if(!this.sm.is_empty()) {
			this.redo_stack.push(this.sm.pop());
		}
	}
	redo=()=>{
		if(!this.redo_stack.is_empty()) {
			this.sm.push(this.redo_stack.pop());
		}
	}
	clear=()=>{
		this.sm.clear();
		this.redo_stack.clear();
	}
	redraw=()=>{
		this.ctx.reset();
		this.ctx.lineWidth = 4;
		this.ctx.lineCap = "round";
		for(let stroke of this.sm.strokes) {
			this.ctx.beginPath();
			if(!stroke.is_empty()) {
				this.ctx.moveTo(stroke.points[0].x, stroke.points[1].x);
			}
			for(let point of stroke.points) {
				this.ctx.lineTo(point.x, point.y);
			}
		}
	}
}
class MonotonicClock {
	private last: number;
	private _now = (): number => {
		return performance.now();
	}
	constructor() {
		this.last = this._now();
	}
	public now = (): number => {
		this.last = Math.max(this.last+0.001, this._now());
		return this.last;
	}
}
class CanvasAnimator {
	private clock: MonotonicClock = new MonotonicClock();
	private readonly sm: StrokeManager;
	private readonly canvas: HTMLCanvasElement;
	private readonly strokeTime: number; // how many millis it takes to draw a stroke
	private readonly strokeDelay: number; // delay in millis between strokes
	private readonly charDelay: number; // delay in millis between loops
	private get totStrokeTime(): number {
		return this.strokeTime + this.strokeDelay;
	}
	private get totTime(): number {
		return this.sm.strokes.length*this.totStrokeTime + this.charDelay;
	}
	private strokei: number = 0;
	private pointi: number = 0;
	private start: number = 0;
	private iid: number|null = null;
	private get ctx(): CanvasRenderingContext2D {
		return this.canvas.getContext("2d") as CanvasRenderingContext2D;
	}
	constructor(canvasId: string, sm: StrokeManager, strokeTime: number, strokeDelay: number, charDelay: number) {
		this.canvas=document.getElementById(canvasId) as HTMLCanvasElement;
		this.sm = sm;
		this.strokeTime = strokeTime;
		this.strokeDelay = strokeDelay;
		this.charDelay = charDelay;
		
		this.start = this.clock.now();
		this.ctx.reset();
		this.iid = setInterval(this.anim, 10);
	}
	drawPointRange = (si: number, p0: number, p1: number) => {
		if(p1 <= p0) {
			return;
		}
		let stroke = this.sm.strokes[si];
		if(p0==0) {
			this.ctx.beginPath();
			p0 = 1;
		}
		this.ctx.moveTo(stroke.points[p0-1].x, stroke.points[p0-1].y);
		for(let i = p0; i < p1; i++) {
			this.ctx.lineTo(stroke.points[i].x, stroke.points[i].y);
		}
		this.ctx.lineWidth = 4;
		this.ctx.lineCap = "round";
		this.ctx.stroke();
	}
	anim=()=>{
		let time = this.clock.now()-this.start;

		while(time > this.totTime) {
			this.ctx.reset();
			this.start = this.start+this.totTime;
			this.strokei=0;
			this.pointi=0;
			time -= this.totTime;
		}

		let nstrokei = Math.min((time/this.totStrokeTime)|0, this.sm.strokes.length-1);
		
		let strokepc = (time-nstrokei*this.totStrokeTime)/this.strokeTime;
		let npointi = Math.min((this.sm.strokes[nstrokei].points.length*strokepc)|0, this.sm.strokes[nstrokei].points.length-1);
		
		npointi+=1;
		while(this.strokei < nstrokei) {
			this.drawPointRange(this.strokei, this.pointi, this.sm.strokes[this.strokei].points.length);
			this.strokei+=1;
			this.pointi=0;
		}
		this.drawPointRange(this.strokei, this.pointi, npointi);
		this.pointi = npointi;
	}
	stop=()=>{
		if(this.iid !== null) {
			clearInterval(this.iid);
			this.iid = null;
		}
	}
}

function tsdraw(): CanvasManager {
	let cm = new CanvasManager("canvas");
	return cm;
}

function tsanim(cm: CanvasManager): CanvasAnimator {
	let am = new CanvasAnimator("canvas", cm.strokes, 100, 200, 200);
	return am;
}

