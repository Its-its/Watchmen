import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ChangesEditorComponent } from './editor.component';

describe('EditorComponent', () => {
  let component: ChangesEditorComponent;
  let fixture: ComponentFixture<ChangesEditorComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ ChangesEditorComponent ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(ChangesEditorComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
