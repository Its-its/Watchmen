import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ListenersComponent } from './listeners.component';

describe('ListenersComponent', () => {
  let component: ListenersComponent;
  let fixture: ComponentFixture<ListenersComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ ListenersComponent ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(ListenersComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
