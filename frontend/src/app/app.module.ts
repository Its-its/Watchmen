import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';


import { HttpClientModule } from '@angular/common/http';


import { MatToolbarModule } from '@angular/material/toolbar';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatListModule } from '@angular/material/list';
import { MatTableModule } from '@angular/material/table';
import { MatGridListModule } from '@angular/material/grid-list';
import { MatChipsModule } from '@angular/material/chips';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatAutocompleteModule } from '@angular/material/autocomplete';


import { BrowserAnimationsModule } from '@angular/platform-browser/animations';


import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { WebsocketService } from "./websocket.service";


import { DashboardComponent } from './dashboard/dashboard.component';
import { ToolbarComponent } from './toolbar/toolbar.component';

import { FeedListComponent } from './feeds/feed-list/feed-list.component';

import { BackgroundService } from './background.service';
import { WebsitesComponent } from './feeds/websites/websites.component';


@NgModule({
	declarations: [
		AppComponent,
		DashboardComponent,
		FeedListComponent,
  		ToolbarComponent,
   		WebsitesComponent,
	],

	imports: [
		HttpClientModule,

		MatAutocompleteModule,
		MatFormFieldModule,
		MatGridListModule,
		MatToolbarModule,
		MatSidenavModule,
		MatButtonModule,
		MatChipsModule,
		MatTableModule,
		MatIconModule,
		MatListModule,

		BrowserModule,
		AppRoutingModule,
		BrowserAnimationsModule
	],

	providers: [
		WebsocketService
	],

	bootstrap: [
		AppComponent
	]
})

export class AppModule {
	constructor(private background: BackgroundService) {
		this.background.init();
	}
}