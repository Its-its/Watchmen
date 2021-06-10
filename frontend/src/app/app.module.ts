import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { MatToolbarModule } from '@angular/material/toolbar';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatListModule } from '@angular/material/list';
import { MatTableModule } from '@angular/material/table';

import { AppRoutingModule } from './app-routing.module';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';

import { AppComponent } from './app.component';
import { DashboardComponent } from './dashboard/dashboard.component';
import { FeedListComponent } from './feed-list/feed-list.component';
import { ToolbarComponent } from './toolbar/toolbar.component';

@NgModule({
	declarations: [
		AppComponent,
		DashboardComponent,
		FeedListComponent,
  		ToolbarComponent
	],

	imports: [
		MatToolbarModule,
		MatSidenavModule,
		MatButtonModule,
		MatTableModule,
		MatIconModule,
		MatListModule,

		BrowserModule,
		AppRoutingModule,
		BrowserAnimationsModule
	],

	providers: [],

	bootstrap: [
		AppComponent
	]
})

export class AppModule { }