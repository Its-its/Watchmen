import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ChangesListComponent } from './list.component';

describe('ListComponent', () => {
  let component: ChangesListComponent;
  let fixture: ComponentFixture<ChangesListComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ ChangesListComponent ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(ChangesListComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
