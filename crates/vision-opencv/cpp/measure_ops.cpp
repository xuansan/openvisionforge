#include "bridge_common.hpp"

static std::string op_ratio(const std::vector<std::string>& args) {
    if (args.size() < 2) {
        fail("usage: ratio <input>");
    }
    cv::Mat gray = to_gray_mat(read_image(args[1]));
    long long white = cv::countNonZero(gray);
    long long total = static_cast<long long>(gray.rows) * gray.cols;
    long long black = total - white;
    double white_ratio = total == 0 ? 0.0 : static_cast<double>(white) / total;
    double black_ratio = total == 0 ? 0.0 : static_cast<double>(black) / total;

    std::ostringstream out;
    out << "{\"width\":" << gray.cols
        << ",\"height\":" << gray.rows
        << ",\"total_pixels\":" << total
        << ",\"black_pixels\":" << black
        << ",\"white_pixels\":" << white
        << ",\"black_ratio\":" << black_ratio
        << ",\"white_ratio\":" << white_ratio
        << "}";
    return out.str();
}

struct ContourItem {
    double area;
    double perimeter;
    cv::Rect rect;
    std::vector<cv::Point> points;
};

static std::string op_contours(const std::vector<std::string>& args) {
    if (args.size() < 4) {
        fail("usage: contours <input> <min_area> <max_contours>");
    }
    cv::Mat gray = to_gray_mat(read_image(args[1]));
    cv::Mat binary;
    cv::threshold(gray, binary, 0.0, 255.0, cv::THRESH_BINARY | cv::THRESH_OTSU);

    std::vector<std::vector<cv::Point>> contours;
    cv::findContours(binary, contours, cv::RETR_EXTERNAL, cv::CHAIN_APPROX_SIMPLE);

    double min_area = std::stod(args[2]);
    int max_contours = std::stoi(args[3]);

    std::vector<ContourItem> items;
    for (const auto& contour : contours) {
        double area = cv::contourArea(contour);
        if (area < min_area) {
            continue;
        }
        items.push_back(ContourItem{
            area,
            cv::arcLength(contour, true),
            cv::boundingRect(contour),
            contour,
        });
    }
    std::sort(items.begin(), items.end(), [](const ContourItem& a, const ContourItem& b) {
        return a.area > b.area;
    });
    if (max_contours > 0 && static_cast<int>(items.size()) > max_contours) {
        items.resize(max_contours);
    }

    std::ostringstream out;
    out << "{\"width\":" << binary.cols
        << ",\"height\":" << binary.rows
        << ",\"contour_count\":" << items.size()
        << ",\"contours\":[";
    for (size_t i = 0; i < items.size(); ++i) {
        const auto& item = items[i];
        if (i > 0) {
            out << ",";
        }
        out << "{\"index\":" << i
            << ",\"x\":" << item.rect.x
            << ",\"y\":" << item.rect.y
            << ",\"width\":" << item.rect.width
            << ",\"height\":" << item.rect.height
            << ",\"area\":" << item.area
            << ",\"perimeter\":" << item.perimeter
            << "}";
    }
    out << "]}";
    return out.str();
}

static std::vector<ContourItem> contour_items(const std::string& input, double min_area, int max_contours) {
    cv::Mat gray = to_gray_mat(read_image(input));
    cv::Mat binary;
    cv::threshold(gray, binary, 0.0, 255.0, cv::THRESH_BINARY | cv::THRESH_OTSU);

    std::vector<std::vector<cv::Point>> contours;
    cv::findContours(binary, contours, cv::RETR_EXTERNAL, cv::CHAIN_APPROX_SIMPLE);

    std::vector<ContourItem> items;
    for (const auto& contour : contours) {
        double area = cv::contourArea(contour);
        if (area < min_area) {
            continue;
        }
        items.push_back(ContourItem{
            area,
            cv::arcLength(contour, true),
            cv::boundingRect(contour),
            contour,
        });
    }
    std::sort(items.begin(), items.end(), [](const ContourItem& a, const ContourItem& b) {
        return a.area > b.area;
    });
    if (max_contours > 0 && static_cast<int>(items.size()) > max_contours) {
        items.resize(max_contours);
    }
    return items;
}

static std::string op_fit_shape(const std::vector<std::string>& args) {
    if (args.size() < 5) {
        fail("usage: fit_shape <input> <shape> <min_area> <max_contours>");
    }
    const std::string& shape = args[2];
    double min_area = std::stod(args[3]);
    int max_contours = std::stoi(args[4]);
    auto items = contour_items(args[1], min_area, max_contours);

    std::ostringstream out;
    out << "{\"shape\":\"" << json_escape(shape)
        << "\",\"count\":" << items.size()
        << ",\"items\":[";
    for (size_t i = 0; i < items.size(); ++i) {
        const auto& item = items[i];
        if (i > 0) {
            out << ",";
        }
        out << "{\"index\":" << i
            << ",\"area\":" << item.area
            << ",\"perimeter\":" << item.perimeter
            << ",\"bbox\":{\"x\":" << item.rect.x
            << ",\"y\":" << item.rect.y
            << ",\"width\":" << item.rect.width
            << ",\"height\":" << item.rect.height
            << "}";

        if (shape == "min_enclosing_circle" || shape == "fit_circle") {
            cv::Point2f center;
            float radius = 0.0f;
            cv::minEnclosingCircle(item.points, center, radius);
            double circle_area = CV_PI * radius * radius;
            double quality = circle_area > 0.0 ? std::min(1.0, item.area / circle_area) : 0.0;
            out << ",\"center_x\":" << center.x
                << ",\"center_y\":" << center.y
                << ",\"radius\":" << radius
                << ",\"diameter\":" << radius * 2.0
                << ",\"quality\":" << quality;
        } else if (shape == "fit_line") {
            cv::Vec4f line;
            cv::fitLine(item.points, line, cv::DIST_L2, 0, 0.01, 0.01);
            double angle = std::atan2(line[1], line[0]) * 180.0 / CV_PI;
            out << ",\"vx\":" << line[0]
                << ",\"vy\":" << line[1]
                << ",\"x\":" << line[2]
                << ",\"y\":" << line[3]
                << ",\"angle_deg\":" << angle
                << ",\"quality\":1";
        } else if (shape == "fit_ellipse") {
            if (item.points.size() >= 5) {
                cv::RotatedRect ellipse = cv::fitEllipse(item.points);
                double ellipse_area = CV_PI * (ellipse.size.width / 2.0) * (ellipse.size.height / 2.0);
                double quality = ellipse_area > 0.0 ? std::min(1.0, item.area / ellipse_area) : 0.0;
                out << ",\"center_x\":" << ellipse.center.x
                    << ",\"center_y\":" << ellipse.center.y
                    << ",\"major_axis\":" << std::max(ellipse.size.width, ellipse.size.height)
                    << ",\"minor_axis\":" << std::min(ellipse.size.width, ellipse.size.height)
                    << ",\"angle_deg\":" << ellipse.angle
                    << ",\"quality\":" << quality;
            } else {
                out << ",\"quality\":0,\"error\":\"fitEllipse requires at least 5 points\"";
            }
        } else {
            fail("unknown fit shape: " + shape);
        }
        out << "}";
    }
    out << "]}";
    return out.str();
}
