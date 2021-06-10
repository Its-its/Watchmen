import {
	Component,
	OnInit, AfterViewInit,
	Input, ViewChildren,
	ElementRef, QueryList
} from '@angular/core';

@Component({
	selector: 'app-toolbar',
	templateUrl: './toolbar.component.html',
	styleUrls: ['./toolbar.component.scss']
})

export class ToolbarComponent implements OnInit, AfterViewInit {
	@Input() items = <[string, string][]>[];

	constructor() {}

	ngOnInit(): void {}

	ngAfterViewInit(): void {
	}
}