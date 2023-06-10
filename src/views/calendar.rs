use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::views::{CalendarPage, DayIndicator};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/calendar.ui")]
    #[properties(wrapper_type=super::Calendar)]
    pub struct Calendar {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[template_child]
        pub day_switcher: TemplateChild<gtk::Box>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Calendar {
        const NAME: &'static str = "Calendar";
        type Type = super::Calendar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                datetime: RefCell::new(glib::DateTime::now_local().unwrap()),
                day_switcher: TemplateChild::default(),
                stack: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for Calendar {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.init_widgets();
        }
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }
    impl WidgetImpl for Calendar {}
    impl BoxImpl for Calendar {}
}

glib::wrapper! {
    pub struct Calendar(ObjectSubclass<imp::Calendar>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl Calendar {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn open_today(&self) {
        let imp = self.imp();
        let today = self.today_datetime();

        let first_indicator = imp
            .day_switcher
            .first_child()
            .and_downcast::<DayIndicator>()
            .unwrap();
        let first_date = first_indicator.datetime();
        let difference = today.difference(&first_date).as_days();
        if difference < 0 {
            for _ in 0..difference.abs() {
                self.switcher_previous();
            }
        } else if difference > 7 {
            for _ in 0..difference - 3 {
                self.switcher_next();
            }
        }

        self.set_page(today);
    }

    pub fn refresh(&self) {
        let imp = self.imp();
        let datetime = self.datetime();
        let name = datetime.format("%F").unwrap();
        let new_page = CalendarPage::new(datetime);
        let pages = imp.stack.observe_children();
        for _ in 0..pages.n_items() {
            imp.stack.remove(&imp.stack.first_child().unwrap());
        }
        imp.stack.add_named(&new_page, Some(&name));
    }

    fn init_widgets(&self) {
        let imp = self.imp();
        let now = self.today_datetime();
        for day in -2..5 {
            let datetime = now.add_days(day).unwrap();
            imp.day_switcher.append(&self.new_day_indicator(datetime));
        }
    }

    fn new_day_indicator(&self, datetime: glib::DateTime) -> DayIndicator {
        let day_indicator = DayIndicator::new(datetime);
        day_indicator.connect_clicked(glib::clone!(@weak self as obj => move |indicator| {
            let datetime = indicator.datetime();
            obj.set_page(datetime);
        }));
        day_indicator
    }

    fn set_page(&self, datetime: glib::DateTime) {
        let imp = self.imp();
        let previous_datetime = self.datetime();
        let name = datetime.format("%F").unwrap();
        let transition = if previous_datetime < datetime {
            gtk::StackTransitionType::SlideLeft
        } else {
            gtk::StackTransitionType::SlideRight
        };

        self.set_datetime(&datetime);
        if imp.stack.child_by_name(&name).is_none() {
            let page = CalendarPage::new(datetime);
            imp.stack.add_named(&page, Some(&name));
        }
        imp.stack.set_visible_child_full(&name, transition);
        self.refresh_indicators_selection();
    }

    fn refresh_indicators_selection(&self) {
        let imp = self.imp();
        let name = imp.stack.visible_child_name().unwrap();
        let indicators = imp.day_switcher.observe_children();
        for i in 0..indicators.n_items() {
            let indicator = indicators.item(i).and_downcast::<DayIndicator>().unwrap();
            if indicator.datetime().format("%F").unwrap() == name {
                indicator.remove_css_class("flat");
            } else {
                indicator.add_css_class("flat");
            }
        }
    }

    fn switcher_next(&self) {
        let imp = self.imp();
        let first_indicator = imp.day_switcher.first_child().unwrap();
        let last_indicator = imp
            .day_switcher
            .last_child()
            .and_downcast::<DayIndicator>()
            .unwrap();
        let datetime = last_indicator.datetime().add_days(1).unwrap();
        let new_indicator = self.new_day_indicator(datetime);
        imp.day_switcher.remove(&first_indicator);
        imp.day_switcher.append(&new_indicator);
        // FIXME: what todo if deleted indicator is active
    }

    fn switcher_previous(&self) {
        let imp = self.imp();
        let first_indicator = imp
            .day_switcher
            .first_child()
            .and_downcast::<DayIndicator>()
            .unwrap();
        let last_indicator = imp.day_switcher.last_child().unwrap();
        let datetime = first_indicator.datetime().add_days(-1).unwrap();
        let new_indicator = self.new_day_indicator(datetime);
        imp.day_switcher.remove(&last_indicator);
        imp.day_switcher.prepend(&new_indicator);
    }

    fn today_datetime(&self) -> glib::DateTime {
        let now = glib::DateTime::now_local().unwrap();
        glib::DateTime::new(
            &glib::TimeZone::local(),
            now.year(),
            now.month(),
            now.day_of_month(),
            0,
            0,
            0.0,
        )
        .unwrap()
    }

    #[template_callback]
    fn handle_next_day_clicked(&self, _: gtk::Button) {
        self.switcher_next();
        self.refresh_indicators_selection();
    }

    #[template_callback]
    fn handle_previous_day_clicked(&self, _: gtk::Button) {
        self.switcher_previous();
        self.refresh_indicators_selection();
    }
}