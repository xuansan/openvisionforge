#include "bridge_common.hpp"

static bool calibration_points(
    const cv::Mat& image,
    const std::string& pattern,
    int columns,
    int rows,
    std::vector<cv::Point2f>& points
) {
    cv::Mat gray;
    if (image.channels() == 1) {
        gray = image;
    } else {
        cv::cvtColor(image, gray, cv::COLOR_BGR2GRAY);
    }
    const cv::Size size(columns, rows);
    if (pattern == "chessboard") {
        const bool found = cv::findChessboardCorners(
            gray,
            size,
            points,
            cv::CALIB_CB_ADAPTIVE_THRESH | cv::CALIB_CB_NORMALIZE_IMAGE
        );
        if (found) {
            cv::cornerSubPix(
                gray,
                points,
                cv::Size(11, 11),
                cv::Size(-1, -1),
                cv::TermCriteria(cv::TermCriteria::EPS | cv::TermCriteria::MAX_ITER, 30, 0.001)
            );
        }
        return found;
    }
    if (pattern == "circle_grid") {
        return cv::findCirclesGrid(gray, size, points, cv::CALIB_CB_SYMMETRIC_GRID);
    }
    fail("calibration pattern must be chessboard or circle_grid");
    return false;
}

static std::string points_json(const std::vector<cv::Point2f>& points) {
    std::ostringstream out;
    out << "[";
    for (size_t i = 0; i < points.size(); ++i) {
        if (i > 0) out << ",";
        out << "[" << points[i].x << "," << points[i].y << "]";
    }
    out << "]";
    return out.str();
}

static std::string op_calibration_detect(const std::vector<std::string>& args) {
    if (args.size() < 5) {
        fail("usage: calibration_detect <input> <chessboard|circle_grid> <columns> <rows>");
    }
    cv::Mat image = read_image(args[1]);
    std::vector<cv::Point2f> points;
    const bool found = calibration_points(
        image,
        args[2],
        std::stoi(args[3]),
        std::stoi(args[4]),
        points
    );
    std::ostringstream out;
    out << "{\"found\":" << (found ? "true" : "false")
        << ",\"image_width\":" << image.cols
        << ",\"image_height\":" << image.rows
        << ",\"point_count\":" << points.size()
        << ",\"points\":" << points_json(points) << "}";
    return out.str();
}

static bool calibration_image_file(const fs::path& path) {
    std::string ext = path.extension().string();
    std::transform(ext.begin(), ext.end(), ext.begin(), ::tolower);
    return ext == ".png" || ext == ".jpg" || ext == ".jpeg" || ext == ".bmp" || ext == ".tif" || ext == ".tiff";
}

static std::string matrix_json(const cv::Mat& matrix) {
    std::ostringstream out;
    out << "[";
    for (int row = 0; row < matrix.rows; ++row) {
        for (int col = 0; col < matrix.cols; ++col) {
            if (row > 0 || col > 0) out << ",";
            out << matrix.at<double>(row, col);
        }
    }
    out << "]";
    return out.str();
}

static std::string op_calibrate_folder(const std::vector<std::string>& args) {
    if (args.size() < 9) {
        fail("usage: calibrate_folder <folder> <pattern> <columns> <rows> <spacing_mm> <min_images> <max_error> <recursive>");
    }
    const fs::path folder(args[1]);
    if (!fs::is_directory(folder)) fail("calibration folder does not exist: " + args[1]);
    const std::string pattern = args[2];
    const int columns = std::stoi(args[3]);
    const int rows = std::stoi(args[4]);
    const double spacing = std::stod(args[5]);
    const int min_images = std::stoi(args[6]);
    const double max_error = std::stod(args[7]);
    const bool recursive = args[8] == "true";
    if (columns < 2 || rows < 2 || spacing <= 0.0 || min_images < 3) fail("invalid calibration parameters");

    std::vector<fs::path> files;
    if (recursive) {
        for (const auto& entry : fs::recursive_directory_iterator(folder)) {
            if (entry.is_regular_file() && calibration_image_file(entry.path())) files.push_back(entry.path());
        }
    } else {
        for (const auto& entry : fs::directory_iterator(folder)) {
            if (entry.is_regular_file() && calibration_image_file(entry.path())) files.push_back(entry.path());
        }
    }
    std::sort(files.begin(), files.end());

    std::vector<std::vector<cv::Point2f>> image_points;
    std::vector<std::vector<cv::Point3f>> object_points;
    cv::Size image_size;
    std::vector<cv::Point3f> object_template;
    for (int row = 0; row < rows; ++row) {
        for (int col = 0; col < columns; ++col) {
            object_template.emplace_back(col * spacing, row * spacing, 0.0f);
        }
    }
    for (const auto& file : files) {
        cv::Mat image = cv::imread(file.string(), cv::IMREAD_COLOR);
        if (image.empty()) continue;
        if (image_size.area() == 0) image_size = image.size();
        if (image.size() != image_size) fail("all calibration images must have the same dimensions");
        std::vector<cv::Point2f> points;
        if (calibration_points(image, pattern, columns, rows, points)) {
            image_points.push_back(points);
            object_points.push_back(object_template);
        }
    }
    if (static_cast<int>(image_points.size()) < min_images) {
        fail("not enough valid calibration images: found " + std::to_string(image_points.size()) + ", required " + std::to_string(min_images));
    }

    cv::Mat camera_matrix = cv::Mat::eye(3, 3, CV_64F);
    cv::Mat distortion;
    std::vector<cv::Mat> rotations;
    std::vector<cv::Mat> translations;
    const double rms = cv::calibrateCamera(
        object_points, image_points, image_size, camera_matrix, distortion, rotations, translations
    );
    double total_error = 0.0;
    for (size_t i = 0; i < object_points.size(); ++i) {
        std::vector<cv::Point2f> projected;
        cv::projectPoints(object_points[i], rotations[i], translations[i], camera_matrix, distortion, projected);
        total_error += cv::norm(image_points[i], projected, cv::NORM_L2) / std::sqrt(static_cast<double>(projected.size()));
    }
    const double mean_error = total_error / object_points.size();
    if (max_error > 0.0 && mean_error > max_error) {
        fail("mean reprojection error exceeds configured maximum");
    }
    std::ostringstream out;
    out << "{\"image_width\":" << image_size.width
        << ",\"image_height\":" << image_size.height
        << ",\"camera_matrix\":" << matrix_json(camera_matrix)
        << ",\"distortion_coefficients\":" << matrix_json(distortion.reshape(1, 1))
        << ",\"rms_error\":" << rms
        << ",\"mean_reprojection_error\":" << mean_error
        << ",\"views_used\":" << image_points.size()
        << ",\"board\":{\"pattern\":\"" << json_escape(pattern)
        << "\",\"columns\":" << columns
        << ",\"rows\":" << rows
        << ",\"spacing_mm\":" << spacing << "}}";
    return out.str();
}

static std::vector<double> parse_numbers(const std::string& text) {
    std::vector<double> values;
    std::stringstream stream(text);
    std::string item;
    while (std::getline(stream, item, ',')) values.push_back(std::stod(item));
    return values;
}

static std::string op_undistort(const std::vector<std::string>& args) {
    if (args.size() < 7) {
        fail("usage: undistort <input> <output> <camera_matrix_csv> <distortion_csv> <calibration_width> <calibration_height>");
    }
    cv::Mat image = read_image(args[1]);
    const auto matrix_values = parse_numbers(args[3]);
    const auto distortion_values = parse_numbers(args[4]);
    if (matrix_values.size() != 9 || distortion_values.size() < 4) fail("invalid camera calibration matrix or distortion coefficients");
    const int calibration_width = std::stoi(args[5]);
    const int calibration_height = std::stoi(args[6]);
    if (image.cols != calibration_width || image.rows != calibration_height) fail("image dimensions do not match camera calibration");
    cv::Mat camera_matrix(3, 3, CV_64F);
    for (int i = 0; i < 9; ++i) camera_matrix.at<double>(i / 3, i % 3) = matrix_values[i];
    cv::Mat distortion(1, static_cast<int>(distortion_values.size()), CV_64F);
    for (size_t i = 0; i < distortion_values.size(); ++i) distortion.at<double>(0, static_cast<int>(i)) = distortion_values[i];
    cv::Mat output;
    cv::undistort(image, output, camera_matrix, distortion);
    write_image(args[2], output);
    return metadata_json(output, args[2]);
}
