import { Component } from '@angular/core';
import { BackgroundService } from 'src/app/background.service';

@Component({
	selector: 'app-editor',
	templateUrl: './editor.component.html',
	styleUrls: ['./editor.component.scss']
})

export class EditorComponent {
	constructor(public background: BackgroundService) {}
}
