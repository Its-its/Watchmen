import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';


import { HttpClientModule } from '@angular/common/http';


import { MatInputModule } from '@angular/material/input';
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
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatMenuModule } from '@angular/material/menu';


import { BrowserAnimationsModule } from '@angular/platform-browser/animations';


import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { WebsocketService } from "./websocket.service";


import { DashboardComponent } from './dashboard/dashboard.component';
import { ToolbarComponent } from './toolbar/toolbar.component';

import { FeedListComponent } from './feeds/feed-list/feed-list.component';
import { FilterEditComponent } from './feeds/filter-edit/filter-edit.component';
import { FilterContainerComponent } from './feeds/filter-edit/filter-container/filter-container.component';

import { BackgroundService } from './background.service';
import { WebsitesComponent } from './feeds/websites/websites.component';


@NgModule({
	declarations: [
		AppComponent,
		DashboardComponent,
		FeedListComponent,
  		ToolbarComponent,
   		WebsitesComponent,
		FilterEditComponent,
		FilterContainerComponent,
	],

	imports: [
		HttpClientModule,

		MatAutocompleteModule,
		MatFormFieldModule,
		MatCheckboxModule,
		MatGridListModule,
		MatToolbarModule,
		MatSidenavModule,
		MatButtonModule,
		MatInputModule,
		MatChipsModule,
		MatTableModule,
		MatIconModule,
		MatListModule,
		MatMenuModule,

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